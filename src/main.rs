mod char_set;
mod cli;
mod compute;
mod io_helper;
mod util;

use std::process;

use clap::Parser;
use compute::generate_random_names;
use log::debug;

use crate::{cli::CliArgs, compute::finalise_names, io_helper::dedup_paths};

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
    let args = CliArgs::parse();
    simple_logger::init_with_level(args.verbosity).map_err(|err| err.to_string())?;
    debug!("{:?}", args);

    #[allow(unused)] // TEMP
    let CliArgs {
        confirm_mode,
        confirm_batch_size,
        extension_mode,
        error_handling_mode,
        name_length,
        name_prefix,
        char_set_selection,
        case,
        verbosity: _,
        files,
    } = args;

    let files_unique = dedup_paths(&files, error_handling_mode)?;

    let char_set = (char_set_selection, case).try_into()?;

    let random_name_pairs = generate_random_names(&files_unique, char_set, name_length)?;

    let finalised_name_pairs = finalise_names(random_name_pairs, name_prefix, extension_mode, error_handling_mode)?;

    Ok(())
}
