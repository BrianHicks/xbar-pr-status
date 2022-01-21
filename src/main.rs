use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Config {}

fn main() {
    if let Err(err) = try_main() {
        println!("{:#?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config = Config::parse();

    println!("{:#?}", config);

    Ok(())
}
