use color_eyre::eyre::Result;
use std::{fmt, fs, path::PathBuf};
use structopt::StructOpt;
use serde_json::Value;
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

  let first: Value =
    serde_json::from_str(&fs::read_to_string(opt.first)?)?;
  let second: Value =
    serde_json::from_str(&fs::read_to_string(opt.second)?)?;

  let mut path = JsonPath::new();
  json(&first, &second, &mut path)
}

fn json(left: &Value, right: &Value, path: &mut JsonPath) -> Result<()> {
  match (left, right) {
    (Value::Object(left_map), Value::Object(right_map)) => {
      for (key, lvalue) in left_map.iter() {
        path.obj(key);
        if let Some(rvalue) = right_map.get(key) {
            json(lvalue, rvalue, path)?;
        } else {
          println!("{} is not in the rvalue", path);
        }
        path.pop();
      }
      for key in right_map.keys() {
        path.obj(key);
        if left_map.get(key).is_none() {
          println!("{} is not in the lvalue", path);
        }
        path.pop();
      }
    }
    (Value::Array(left_array), Value::Array(right_array)) => {
      //let order_matters = true;
      path.array(0);
      for (lvalue, rvalue) in left_array.iter().zip(right_array) {
        json(lvalue, rvalue, path)?;
        path.inc_array();
      }
      path.pop();

      let llen = left_array.len();
      let rlen = right_array.len();

      if llen != rlen {
        if llen > rlen {
          println!("The lvalue {} has {} more elements than the rvalue", path, llen - rlen);

        } else {
          println!("The rvalue {} has {} more elements than the lvalue", path, rlen - llen);
        }
      }
    }
    (Value::String(lvalue), Value::String(rvalue)) => {
      if lvalue != rvalue {
        println!("{} => {} <> {}", path, lvalue, rvalue);
      }
    },
    (Value::Number(lvalue), Value::Number(rvalue)) => {
      if lvalue != rvalue {
        println!("{} => {} <> {}", path, lvalue, rvalue);
      }
    },
    (Value::Bool(lvalue), Value::Bool(rvalue)) => {
      if lvalue != rvalue {
        println!("{} => {} <> {}", path, lvalue, rvalue);
      }
    },
    (Value::Null, Value::Null) => {}
    (lvalue, rvalue) => {
        println!("{} => {} <> {}", path, lvalue, rvalue);
    }
  }
  Ok(())
}
struct JsonPath {
  path: Vec<JsonType>
}

enum JsonType {
  Object(String),
  Array(usize),
}

impl JsonPath {
  fn new() -> Self {
    Self { path: Vec::new() }
  }
  fn obj<T: Into<String>>(&mut self, input: T) {
    self.path.push(JsonType::Object(input.into()));
  }
  fn array(&mut self, input: usize) {
    self.path.push(JsonType::Array(input));
  }
  fn pop(&mut self) {
    self.path.pop();
  }
  fn inc_array(&mut self) {
    if let Some(v) = self.path.last_mut() {
      let x = match v {
        JsonType::Array(i) => *i + 1,
        _ => panic!("Oh no this not array"),
      };
      *v = JsonType::Array(x);
    }
  }
}

impl fmt::Display for JsonPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, value) in self.path.iter().enumerate() {
      match value {
        JsonType::Array(index) => write!(f, "[{}]", index)?,
        JsonType::Object(field) => if i == 0 {
          write!(f, "{}", field)?
        } else {
          write!(f, ".{}", field)?
        },
      }
    }
    Ok(())
  }
}
