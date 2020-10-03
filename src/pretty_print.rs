use crate::{Diff, DiffKind};
use color_eyre::eyre::Result;
use prettytable::{cell, row, Cell, Table};
use std::path::Path;

pub fn print_tables<T, P>(
    iter: T,
    left: P,
    right: P,
    print_string: Option<&mut String>,
) -> Result<()>
where
    T: Iterator<Item = Diff>,
    P: AsRef<Path>,
{
    let mut diffs = iter.collect::<Vec<Diff>>();
    diffs.sort_unstable_by(|left, right| left.path.partial_cmp(&right.path).unwrap());
    let mut table = Table::new();
    table.add_row(row![
        "JSON Path",
        left.as_ref().display(),
        right.as_ref().display(),
        "Diff Kind"
    ]);
    for diff in diffs {
        let path = Cell::new(&diff.path.to_string());
        match diff.kind {
            DiffKind::LvalueDNE => {
                let l_dne = Cell::new("DNE");
                let r_dne = Cell::new("");
                table.add_row(row![path, l_dne, r_dne, "Left Value DNE"]);
            }
            DiffKind::RvalueDNE => {
                let l_dne = Cell::new("");
                let r_dne = Cell::new("DNE");
                table.add_row(row![path, l_dne, r_dne, "Right Value DNE"]);
            }
            DiffKind::DiffValue(l, r) => {
                table.add_row(row![path, &l.to_string(), &r.to_string(), "Value Diff"]);
            }
            DiffKind::LMoreElements(num) => {
                let l = Cell::new(&num.to_string());
                let r = Cell::new("");
                table.add_row(row![path, l, r, "Left More Elements"]);
            }
            DiffKind::RMoreElements(num) => {
                let l = Cell::new("");
                let r = Cell::new(&num.to_string());
                table.add_row(row![path, l, r, "Right More Elements"]);
            }
        }
    }
    if let Some(string) = print_string {
        *string = table.to_string();
    } else {
        table.printstd();
    }
    Ok(())
}
