use color_eyre::eyre::Result;
use crossbeam::crossbeam_channel::{self, Sender};
use im::Vector;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use serde_json::Value;
use std::{fmt, fs, path::PathBuf};
use structopt::StructOpt;
use tracing_subscriber;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    first: PathBuf,
    #[structopt(parse(from_os_str))]
    second: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    let first: Value = serde_json::from_str(&fs::read_to_string(opt.first)?)?;
    let second: Value = serde_json::from_str(&fs::read_to_string(opt.second)?)?;

    let path = JsonPath::new();
    let (tx, rx) = crossbeam_channel::unbounded();
    json(&first, &second, path, tx);

    for diff in rx.iter() {
        println!("{}", diff);
    }
    Ok(())
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

struct Diff {
    path: JsonPath,
    kind: DiffKind,
}

enum DiffKind {
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

#[derive(Clone)]
struct JsonPath {
    path: Vector<JsonType>,
}

#[derive(Clone)]
enum JsonType {
    Object(String),
    Array(usize),
}

impl JsonPath {
    fn new() -> Self {
        Self {
            path: Vector::new(),
        }
    }
    fn obj<T: Into<String>>(&mut self, input: T) {
        self.path.push_back(JsonType::Object(input.into()));
    }
    fn array(&mut self, input: usize) {
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
