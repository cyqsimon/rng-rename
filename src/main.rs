mod cli;

use clap::Parser;

use crate::cli::CliArgs;

fn main() {
    let _args = CliArgs::parse();
    println!("Hello, world!");
}
