use color_eyre::eyre::Result;
use crossbeam::crossbeam_channel::Receiver;
use divergent::{pretty_print::print_tables, *};
use std::path::PathBuf;

#[test]
fn test_json_visual() -> Result<()> {
    let test_fixture = include_str!("visual/json1.txt");
    let rx = json(1, 2)?;
    let mut table = String::new();
    print_tables(rx.iter(), json_path(1), json_path(2), Some(&mut table))?;
    assert_eq!(test_fixture, table);
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
