use std::path::PathBuf;

use clap::Parser;
use cidr_calc::eval::{eval, format, Value, eval_stmt, Scope};
use cidr_calc::parser::{parse, parse_single};
use rustyline::DefaultEditor;

#[derive(Parser)]
struct Args {
    input: Option<PathBuf>, // TODO: option, none is repl
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.input.is_none() {
        return repl();
    }

    let input = args.input.unwrap();
    let content = std::fs::read_to_string(&input)?;
    let parsed = parse(&content)?;
    let evaled = eval(&parsed)?;
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

fn repl() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    let mut scope = Scope::default();
    loop {
        let line = rl.readline("> ");
        match line {
            Ok(line) => {
                if line == "/s" {
                    println!("In scope: {}", scope.keys().collect::<Vec<_>>().join(", "));
                } else {
                    let evaled: anyhow::Result<_> = (|| {
                        let stmt = parse_single(&line)?;
                        let (v, s) = eval_stmt(&stmt, scope.clone())?;
                        scope = s;
                        Ok(v)
                    })();
                    match evaled {
                        Ok(Value::Unit) => {
                            continue;
                        }
                        Ok(v) => {
                            println!("[");
                            for l in format(&v) {
                                println!("\t{}", l);
                            }
                            println!("]");
                        }
                        Err(e) => {
                            println!("Evaluation error:");
                            println!("{}", e);
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                return Ok(())
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                return Err(anyhow::anyhow!("Interrupted"));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Unexpected error: {}", e));
            }
        }
    }
}
