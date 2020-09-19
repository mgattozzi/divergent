use color_eyre::eyre::Result;
use crossbeam::crossbeam_channel::{self, Receiver, Sender};
use im::Vector;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use serde_json::Value;
use std::hash::{Hash, Hasher};
use std::{fmt, fs, path::Path};

pub fn run_json<P: AsRef<Path>>(left_path: P, right_path: P) -> Result<Receiver<Diff>> {
    let first: Value = serde_json::from_str(&fs::read_to_string(left_path.as_ref())?)?;
    let second: Value = serde_json::from_str(&fs::read_to_string(right_path.as_ref())?)?;
    let path = JsonPath::new();
    let (tx, rx) = crossbeam_channel::unbounded();
    json(&first, &second, path, tx);
    Ok(rx)
}

fn json(left: &Value, right: &Value, path: JsonPath, tx: Sender<Diff>) {
    match (left, right) {
        (Value::Object(left_map), Value::Object(right_map)) => {
            left_map.iter().par_bridge().for_each(|(key, lvalue)| {
                let mut path = path.clone();
                path.obj(key);
                if let Some(rvalue) = right_map.get(key) {
                    json(lvalue, rvalue, path, tx.clone());
                } else {
                    tx.send(Diff {
                        path,
                        kind: DiffKind::RvalueDNE,
                    })
                    .unwrap();
                }
            });
            right_map.keys().par_bridge().for_each(|rvalue| {
                let mut path = path.clone();
                path.obj(rvalue);
                if left_map.get(rvalue).is_none() {
                    tx.send(Diff {
                        path,
                        kind: DiffKind::LvalueDNE,
                    })
                    .unwrap();
                }
            });
        }
        (Value::Array(left_array), Value::Array(right_array)) => {
            //let order_matters = true;
            left_array
                .iter()
                .zip(right_array)
                .enumerate()
                .par_bridge()
                .for_each(|(i, (lvalue, rvalue))| {
                    let mut path = path.clone();
                    path.array(i);
                    json(lvalue, rvalue, path.clone(), tx.clone());
                });

            let llen = left_array.len();
            let rlen = right_array.len();

            if llen != rlen {
                if llen > rlen {
                    tx.send(Diff {
                        path: path.clone(),
                        kind: DiffKind::LMoreElements(llen - rlen),
                    })
                    .unwrap();
                } else {
                    tx.send(Diff {
                        path: path.clone(),
                        kind: DiffKind::RMoreElements(rlen - llen),
                    })
                    .unwrap();
                }
            }
        }
        (lvalue, rvalue) => {
            if lvalue != rvalue {
                tx.send(Diff {
                    path: path.clone(),
                    kind: DiffKind::DiffValue(lvalue.clone(), rvalue.clone()),
                })
                .unwrap();
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Diff {
    pub path: JsonPath,
    pub kind: DiffKind,
}

#[derive(PartialEq, Eq, Debug)]
pub enum DiffKind {
    LvalueDNE,
    RvalueDNE,
    DiffValue(Value, Value),
    LMoreElements(usize),
    RMoreElements(usize),
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DiffKind::LvalueDNE => write!(f, "{} is not in the lvalue", self.path),
            DiffKind::RvalueDNE => write!(f, "{} is not in the rvalue", self.path),
            DiffKind::DiffValue(lvalue, rvalue) => {
                write!(f, "{} => {} <> {}", self.path, lvalue, rvalue)
            }
            DiffKind::LMoreElements(n) => write!(
                f,
                "The lvalue {} has {} more elements than the rvalue",
                self.path, n
            ),
            DiffKind::RMoreElements(n) => write!(
                f,
                "The rvalue {} has {} more elements than the lvalue",
                self.path, n
            ),
        }
    }
}

impl Hash for DiffKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        fn hash_value<H: Hasher>(v: &Value, state: &mut H) {
            match v {
                Value::Null => {
                    let null: Option<usize> = None;
                    null.hash(state);
                }
                Value::Bool(b) => b.hash(state),
                Value::Number(n) => {
                    if let Some(ni) = n.as_i64() {
                        ni.hash(state);
                    } else if let Some(nu) = n.as_u64() {
                        nu.hash(state);
                    } else {
                        n.as_f64().unwrap().to_be_bytes().hash(state)
                    }
                }
                Value::String(s) => s.hash(state),
                Value::Array(a) => a.iter().for_each(|i| hash_value(i, state)),
                Value::Object(o) => o.iter().for_each(|(k, v)| {
                    k.hash(state);
                    hash_value(v, state);
                }),
            }
        }
        match self {
            Self::DiffValue(l, r) => {
                hash_value(l, state);
                hash_value(r, state);
            }
            Self::LvalueDNE => "LvalueDNE".hash(state),
            Self::RvalueDNE => "RvalueDNE".hash(state),
            Self::LMoreElements(n) => {
                "LMoreElements".hash(state);
                n.hash(state);
            }
            Self::RMoreElements(n) => {
                "RMoreElements".hash(state);
                n.hash(state);
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct JsonPath {
    path: Vector<JsonType>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum JsonType {
    Object(String),
    Array(usize),
}

impl JsonPath {
    pub fn new() -> Self {
        Self {
            path: Vector::new(),
        }
    }
    pub fn obj<T: Into<String>>(&mut self, input: T) {
        self.path.push_back(JsonType::Object(input.into()));
    }
    pub fn array(&mut self, input: usize) {
        self.path.push_back(JsonType::Array(input));
    }
}

impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, value) in self.path.iter().enumerate() {
            match value {
                JsonType::Array(index) => write!(f, "[{}]", index)?,
                JsonType::Object(field) => {
                    if i == 0 {
                        write!(f, "{}", field)?
                    } else {
                        write!(f, ".{}", field)?
                    }
                }
            }
        }
        Ok(())
    }
}
