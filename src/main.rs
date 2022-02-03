mod char_set;
mod cli;
mod compute;
mod io_helper;

use std::process;

use clap::Parser;
use compute::generate_random_names;
use simple_logger::SimpleLogger;

use crate::{cli::CliArgs, io_helper::dedup_paths};

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

    let CliArgs {
        confirm_mode,
        confirm_batch_size,
        no_extension,
        error_handling_mode,
        name_length,
        name_prefix,
        char_set_selection,
        case,
        files,
    } = CliArgs::parse();

    let files_unique = dedup_paths(&files, error_handling_mode).map_err(|err| err.to_string())?;

    let char_set = (char_set_selection, case).try_into()?;

    let pairs = generate_random_names(&files_unique, char_set, name_length).map_err(|err| err.to_string())?;
    println!("{:#?}", pairs);

    Ok(())
}
