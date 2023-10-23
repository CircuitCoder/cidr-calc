use clap::Parser;

mod parser;
mod eval;
mod data;

#[derive(Parser)]
struct Args {
    repl: bool,
}

fn main() {
    println!("Hello, world!");
}
