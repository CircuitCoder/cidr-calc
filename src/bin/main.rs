use std::path::PathBuf;

use clap::Parser;
use cidr_calc::eval::{eval, format, Value};
use cidr_calc::parser::parse;

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
    println!("{:?}", evaled);
    for value in evaled {
        match value {
            Value::Unit => {},
            _ => {
                println!("[{}]", format(&value).collect::<Vec<_>>().join(","))
            }
        }
    }

    Ok(())
}
