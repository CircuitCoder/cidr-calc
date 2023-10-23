use std::path::PathBuf;

use clap::Parser;
use eval::{eval, format};
use parser::parse;

mod parser;
mod eval;
mod data;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf, // TODO: option, none is repl
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let content = std::fs::read_to_string(&args.input)?;
    let parsed = parse(&content)?;
    let evaled = eval(&parsed)?;
    for row in format(&evaled) {
        println!("{}", row);
    }
    Ok(())
}
