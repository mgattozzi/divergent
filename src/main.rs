use color_eyre::eyre::Result;
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

    for diff in divergent::run_json(&opt.first, &opt.second)?.iter() {
        println!("{}", diff);
    }

    Ok(())
}
