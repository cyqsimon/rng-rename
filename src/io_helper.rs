use std::{
    fmt, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use ansi_term::Colour;
use dialoguer::Input;
use itertools::Itertools;
use log::{debug, info, trace};

use crate::{
    cli::{ConfirmMode, ErrorHandlingMode},
    util::{error_prompt, OnErrorResponse},
};

#[derive(Debug)]
pub enum DedupError {
    IOError(io::Error),
    UserHalt,
}
impl fmt::Display for DedupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::IOError(err) => err.to_string(),
            Self::UserHalt => "user halt".into(),
        };
        write!(f, "Failed during canonicalise & dedup step: {}", repr)
    }
}
impl From<DedupError> for String {
    fn from(err: DedupError) -> Self {
        err.to_string()
    }
}
impl From<io::Error> for DedupError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

/// Canonicalise all paths, then deduplicate them.
///
/// The behaviour when an error is encountered depends on `err_mode`.
pub fn dedup_paths<P>(files: &[P], err_mode: ErrorHandlingMode) -> Result<Vec<PathBuf>, DedupError>
where
    P: AsRef<Path>,
{
    let mut canonicalised = vec![];

    for path in files {
        'retry: loop {
            let abs_path_res = path.as_ref().canonicalize();
            match (abs_path_res, err_mode) {
                (Ok(abs_path), _) => {
                    trace!("Canonicalised path {:?} into {:?}.", path.as_ref(), abs_path);
                    canonicalised.push(abs_path);
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Ignore) => {
                    debug!("Error canonicalising path {:?}: {}. Ignoring.", path.as_ref(), err);
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Warn) => {
                    debug!("Error canonicalising path {:?}: {}. Prompting.", path.as_ref(), err);

                    println!(
                        "Error canonicalising path {}: {}",
                        Colour::Red.paint(format!("{:?}", path.as_ref())),
                        err
                    );
                    let user_response = error_prompt("What to do with this path?", Some(OnErrorResponse::Skip))?;
                    trace!("User selected \"{}\"", user_response);

                    match user_response {
                        OnErrorResponse::Skip => break 'retry,
                        OnErrorResponse::Retry => continue 'retry,
                        OnErrorResponse::Halt => Err(DedupError::UserHalt)?,
                    }
                }
                (Err(err), ErrorHandlingMode::Halt) => {
                    debug!("Error canonicalising path {:?}: {}. Failing.", path.as_ref(), err);
                    Err(err)?;
                }
            }
        }
    }

    canonicalised.dedup();
    Ok(canonicalised)
}

#[derive(Debug)]
pub enum RenameError {
    IOError(io::Error),
    UserHalt,
}
impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::IOError(err) => err.to_string(),
            Self::UserHalt => "user halt".into(),
        };
        write!(f, "Failed during rename step: {}", repr)
    }
}
impl From<RenameError> for String {
    fn from(err: RenameError) -> Self {
        err.to_string()
    }
}
impl From<io::Error> for RenameError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

/// Perform the rename using the provided `path`-`new name` pairs.
///
/// The behaviour when an error is encountered depends on `err_mode`.
pub fn rename_files<P, S>(
    pairs_list: &[(P, S)],
    confirm_mode: ConfirmMode,
    confirm_batch_size: usize,
    err_mode: ErrorHandlingMode,
) -> Result<(), RenameError>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    match confirm_mode {
        ConfirmMode::None => rename_files_no_confirm(pairs_list, err_mode),
        ConfirmMode::Batch => rename_files_confirm(pairs_list, confirm_batch_size, err_mode),
        ConfirmMode::Each => rename_files_confirm(pairs_list, 1, err_mode),
    }
}

fn rename_files_no_confirm<P, S>(pairs_list: &[(P, S)], err_mode: ErrorHandlingMode) -> Result<(), RenameError>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let mut success_count = 0;

    debug!("Renaming files without confirmation.");
    for (path, new_name) in pairs_list {
        let path = path.as_ref();
        let new_name = new_name.as_ref();
        'retry: loop {
            let rename_res = try_rename(path, new_name, err_mode);
            match (rename_res, err_mode) {
                (Ok(_), _) => {
                    trace!("Rename from {:?} to {} successful.", path, new_name);
                    success_count += 1;
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Ignore) => {
                    debug!("Failed to rename {:?} to {}: {}, ignoring.", path, new_name, err);
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Warn) => {
                    debug!("Failed to rename {:?} to {}: {}. Prompting.", path, new_name, err);
                    println!(
                        "Failed to rename {:?} to {}: {}",
                        Colour::Red.paint(format!("{:?}", path)),
                        Colour::Red.paint(new_name),
                        err
                    );
                    let user_response = error_prompt("What to do with this file?", Some(OnErrorResponse::Skip))?;
                    trace!("User selected \"{}\"", user_response);

                    match user_response {
                        OnErrorResponse::Skip => break 'retry,
                        OnErrorResponse::Retry => continue 'retry,
                        OnErrorResponse::Halt => Err(RenameError::UserHalt)?,
                    }
                }
                (Err(err), ErrorHandlingMode::Halt) => {
                    debug!("Failed to rename {:?} to {}: {}. Halting.", path, new_name, err);
                    Err(err)?;
                }
            }
        }
    }

    info!("Successfully renamed {} files", success_count);
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BatchConfirmResponse {
    Proceed,
    Skip,
    Halt,
}
impl fmt::Display for BatchConfirmResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BatchConfirmResponse::*;
        let repr = match self {
            Proceed => "proceed",
            Skip => "skip",
            Halt => "halt",
        };
        write!(f, "{}", repr)
    }
}
impl FromStr for BatchConfirmResponse {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BatchConfirmResponse::*;
        Ok(match s.to_lowercase().as_str() {
            "p" | "proceed" => Proceed,
            "s" | "skip" => Skip,
            "h" | "halt" => Halt,
            other => Err(format!("\"{}\" is not a valid response", other))?,
        })
    }
}

fn rename_files_confirm<P, S>(
    pairs_list: &[(P, S)],
    batch_size: usize,
    err_mode: ErrorHandlingMode,
) -> Result<(), RenameError>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let mut success_count = 0;

    debug!("Renaming files with confirmation and batch size of {}.", batch_size);
    let batch_count = ((pairs_list.len() as f64) / (batch_size as f64)).ceil() as usize;
    'batch: for (batch_idx, batch) in pairs_list.chunks(batch_size).enumerate() {
        trace!("Processing batch {}.", batch_idx);

        // confirm batch
        println!(
            "Batch {}/{}:",
            Colour::Yellow.paint(format!("#{}", batch_idx + 1)),
            Colour::Green.paint(batch_count.to_string())
        );
        let batch_info_text = batch
            .iter()
            .map(|(path, new_name)| {
                format!(
                    "\t{} -> {}",
                    Colour::Yellow.paint(format!("{:?}", path.as_ref())),
                    Colour::Green.paint(format!("\"{}\"", new_name.as_ref()))
                )
            })
            .join("\n");
        println!("{}", batch_info_text);

        use Colour::Green;
        let prompt_text = format!(
            "Confirm batch? You can {}({}), {}({}), or {}({})",
            Green.paint("proceed"),
            Green.paint("p"),
            Green.paint("skip"),
            Green.paint("s"),
            Green.paint("halt"),
            Green.paint("h")
        );
        let user_response = Input::new()
            .default(BatchConfirmResponse::Proceed)
            .with_prompt(prompt_text)
            .interact()?;
        trace!("User selected \"{}\"", user_response);

        match user_response {
            BatchConfirmResponse::Proceed => {} // fall through
            BatchConfirmResponse::Skip => continue 'batch,
            BatchConfirmResponse::Halt => Err(RenameError::UserHalt)?,
        }

        // rename batch
        for (path, new_name) in batch {
            let path = path.as_ref();
            let new_name = new_name.as_ref();
            'retry: loop {
                let rename_res = try_rename(path, new_name, err_mode);
                match (rename_res, err_mode) {
                    (Ok(_), _) => {
                        trace!("Rename from {:?} to {} successful.", path, new_name);
                        success_count += 1;
                        break 'retry;
                    }
                    (Err(err), ErrorHandlingMode::Ignore) => {
                        debug!("Failed to rename {:?} to {}: {}, ignoring.", path, new_name, err);
                        break 'retry;
                    }
                    (Err(err), ErrorHandlingMode::Warn) => {
                        debug!("Failed to rename {:?} to {}: {}. Prompting.", path, new_name, err);
                        println!(
                            "Failed to rename {:?} to {}: {}",
                            Colour::Red.paint(format!("{:?}", path)),
                            Colour::Red.paint(new_name),
                            err
                        );
                        let user_response = error_prompt("What to do with this file?", Some(OnErrorResponse::Skip))?;
                        trace!("User selected \"{}\"", user_response);

                        match user_response {
                            OnErrorResponse::Skip => break 'retry,
                            OnErrorResponse::Retry => continue 'retry,
                            OnErrorResponse::Halt => Err(RenameError::UserHalt)?,
                        }
                    }
                    (Err(err), ErrorHandlingMode::Halt) => {
                        debug!("Failed to rename {:?} to {}: {}. Halting.", path, new_name, err);
                        Err(err)?;
                    }
                }
            }
        }
    }

    info!("Successfully renamed {} files", success_count);
    Ok(())
}

fn try_rename(path: &Path, new_name: &str, err_mode: ErrorHandlingMode) -> io::Result<()> {
    // TEMP
    println!("Path: {:?}, Name: {}, Mode: {:?}", path, new_name, err_mode);

    // TODO
    // 'retry: loop {
    //     let rename_res: Result<(), ()> = Ok(todo!());
    //     match (rename_res, err_mode) {
    //         (Ok(_), _) => trace!("Renamed {:?} to {}", path, new_name),
    //         (Err(err), ErrorHandlingMode::Ignore) => todo!(),
    //         (Err(err), ErrorHandlingMode::Warn) => todo!(),
    //         (Err(err), ErrorHandlingMode::Halt) => todo!(),
    //     }
    // }

    Ok(())
}
