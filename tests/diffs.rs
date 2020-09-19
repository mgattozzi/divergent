use color_eyre::eyre::Result;
use crossbeam::crossbeam_channel::Receiver;
use divergent::*;
use serde_json::Value;
use std::{collections::HashSet, path::PathBuf};

#[test]
fn test_json_1_2() -> Result<()> {
    let mut set = HashSet::new();

    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("bar");
            path
        },
        kind: DiffKind::DiffValue("C".into(), "B".into()),
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("blah");
            path
        },
        kind: DiffKind::RvalueDNE,
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("bool2");
            path
        },
        kind: DiffKind::DiffValue(true.into(), false.into()),
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("weird");
            path
        },
        kind: DiffKind::DiffValue(
            Value::Null.into(),
            Value::Object(serde_json::Map::new()).into(),
        ),
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("array");
            path.array(2);
            path
        },
        kind: DiffKind::DiffValue(2.into(), 3.into()),
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("baz");
            path.obj("zed");
            path
        },
        kind: DiffKind::DiffValue("bleh".into(), "pie".into()),
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("array");
            path
        },
        kind: DiffKind::LMoreElements(1),
    });
    set.insert(Diff {
        path: {
            let mut path = JsonPath::new();
            path.obj("bleh");
            path
        },
        kind: DiffKind::LvalueDNE,
    });

    let rx = json(1, 2)?;

    for diff in rx.iter() {
        assert!(set.contains(&diff));
    }

    Ok(())
}

fn json(left: usize, right: usize) -> Result<Receiver<Diff>> {
    run_json(json_path(left), json_path(right))
}

fn json_path(num: usize) -> PathBuf {
    PathBuf::new()
        .join("tests")
        .join("json")
        .join(&format!("test{}.json", num))
}
