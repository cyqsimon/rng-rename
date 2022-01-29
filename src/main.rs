mod char_set;
mod cli;
mod compute;

use std::process;

use clap::Parser;
use compute::generate_names;
use simple_logger::SimpleLogger;

use crate::cli::CliArgs;

fn main() {
    match main_impl() {
        Ok(_) => {}
        Err(err) => {
            println!("{}", err);
            process::exit(1)
        }
    };
}

fn main_impl() -> Result<(), String> {
    SimpleLogger::new().init().map_err(|err| err.to_string())?;

    let args = CliArgs::parse();
    println!("{:?}", args);

    let char_set = (args.char_set_selection, args.case).try_into()?;

    let pairs = generate_names(&args.files, char_set, args.name_length).map_err(|err| err.to_string())?;
    println!("{:#?}", pairs);

    Ok(())
}
