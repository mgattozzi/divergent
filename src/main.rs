use color_eyre::eyre::Result;
use divergent::pretty_print::print_tables;
use std::path::PathBuf;
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

    let print_std = None;
    print_tables(
        divergent::run_json(&opt.first, &opt.second)?.iter(),
        &opt.first,
        &opt.second,
        print_std,
    )?;

    Ok(())
}
