// Using enum unqualified is bad form. See https://youtu.be/8j_FbjiowvE?t=97.
#![deny(clippy::enum_glob_use)]

mod char_set;
mod cli;
mod compute;
mod io_helper;
mod util;

use clap::{crate_name, CommandFactory, Parser};
use compute::generate_random_names;
use log::debug;
use yansi::Paint;

use crate::{
    cli::{CliArgs, SubCmd},
    compute::finalise_names,
    io_helper::{dedup_paths, rename_files},
};

fn main() -> Result<(), String> {
    // set conditional colourisation
    yansi::whenever(yansi::Condition::TTY_AND_COLOR);

    let args = CliArgs::parse();
    if let Some(lvl) = args.verbosity.log_level() {
        simple_logger::init_with_level(lvl).map_err(|err| err.to_string())?;
    }
    debug!("{args:?}");

    let CliArgs {
        sub_cmd,
        confirm_mode,
        confirm_batch_size,
        dry_run,
        extension_mode_selection,
        static_ext,
        error_handling_mode,
        force_generation_strategy,
        name_length,
        name_prefix,
        name_suffix,
        char_set_selection,
        custom_chars,
        case,
        verbosity: _,
        files,
    } = args;

    if let Some(sub_cmd) = sub_cmd {
        match sub_cmd {
            SubCmd::Complete { shell_type } => {
                clap_complete::generate(
                    shell_type,
                    &mut CliArgs::command(),
                    crate_name!(),
                    &mut std::io::stdout(),
                );
                return Ok(());
            }
        }
    }

    if dry_run {
        println!("You are in {}. Your files will not be touched.", "DRY RUN MODE".red());
    }

    let files_unique = dedup_paths(&files, error_handling_mode)?;

    let char_set = (char_set_selection, custom_chars, case).try_into()?;
    debug!("Character set is {char_set}");
    let random_name_pairs = generate_random_names(&files_unique, char_set, name_length, force_generation_strategy)?;

    let extension_mode = (extension_mode_selection, static_ext).try_into()?;
    debug!("Extension mode is {extension_mode}");
    let finalised_name_pairs = finalise_names(
        random_name_pairs,
        name_prefix,
        name_suffix,
        extension_mode,
        error_handling_mode,
    )?;

    let success_count = rename_files(
        &finalised_name_pairs,
        dry_run,
        confirm_mode,
        confirm_batch_size,
        error_handling_mode,
    )?;

    println!(
        "Renamed {} files{}. Done.",
        success_count.green(),
        if dry_run {
            format!(" ({})", "DRY RUN".red())
        } else {
            "".into()
        }
    );

    Ok(())
}
