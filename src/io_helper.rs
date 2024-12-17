use std::{
    fmt, fs, io,
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
    DialoguerError(dialoguer::Error),
    UserHalt,
}
impl From<io::Error> for DedupError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}
impl From<dialoguer::Error> for DedupError {
    fn from(err: dialoguer::Error) -> Self {
        Self::DialoguerError(err)
    }
}
impl fmt::Display for DedupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::IOError(err) => err.to_string(),
            Self::DialoguerError(err) => err.to_string(),
            Self::UserHalt => "user halt".into(),
        };
        write!(f, "Failed during canonicalise & dedup step: {repr}")
    }
}
impl From<DedupError> for String {
    fn from(err: DedupError) -> Self {
        err.to_string()
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
        let path = path.as_ref();
        'retry: loop {
            let abs_path_res = path.canonicalize();
            match (abs_path_res, err_mode) {
                (Ok(abs_path), _) => {
                    trace!("Canonicalised {path:?} into {abs_path:?}.");
                    canonicalised.push(abs_path);
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Ignore) => {
                    debug!("Error canonicalising path {path:?}: {err}. Ignoring.");
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Warn) => {
                    debug!("Error canonicalising path {path:?}: {err}. Prompting.");

                    println!(
                        "Error canonicalising path {}: {err}",
                        Colour::Red.paint(format!("{path:?}")),
                    );
                    let user_response = error_prompt("What to do with this path?", Some(OnErrorResponse::Skip))?;
                    trace!("User selected \"{user_response}\"");

                    match user_response {
                        OnErrorResponse::Skip => break 'retry,
                        OnErrorResponse::Retry => continue 'retry,
                        OnErrorResponse::Halt => Err(DedupError::UserHalt)?,
                    }
                }
                (Err(err), ErrorHandlingMode::Halt) => {
                    debug!("Error canonicalising path {path:?}: {err}. Failing.");
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
    DialoguerError(dialoguer::Error),
    UserHalt,
}
impl From<io::Error> for RenameError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}
impl From<dialoguer::Error> for RenameError {
    fn from(err: dialoguer::Error) -> Self {
        Self::DialoguerError(err)
    }
}
impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::IOError(err) => err.to_string(),
            Self::DialoguerError(err) => err.to_string(),
            Self::UserHalt => "user halt".into(),
        };
        write!(f, "Failed during rename step: {repr}")
    }
}
impl From<RenameError> for String {
    fn from(err: RenameError) -> Self {
        err.to_string()
    }
}

/// Perform the rename using the provided `path`-`new name` pairs.
/// Returns the number of successfully renamed files.
///
/// The behaviour when an error is encountered depends on `err_mode`.
pub fn rename_files<P, S>(
    pairs_list: &[(P, S)],
    dry_run: bool,
    confirm_mode: ConfirmMode,
    confirm_batch_size: usize,
    err_mode: ErrorHandlingMode,
) -> Result<usize, RenameError>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    match confirm_mode {
        ConfirmMode::None => rename_files_no_confirm(pairs_list, dry_run, err_mode),
        ConfirmMode::Batch => rename_files_confirm(pairs_list, dry_run, confirm_batch_size, err_mode),
        ConfirmMode::Each => rename_files_confirm(pairs_list, dry_run, 1, err_mode),
    }
}

fn rename_files_no_confirm(
    pairs_list: &[(impl AsRef<Path>, impl AsRef<str>)],
    dry_run: bool,
    err_mode: ErrorHandlingMode,
) -> Result<usize, RenameError> {
    let mut success_count = 0;

    debug!("Renaming files without confirmation.");
    for (path, new_name) in pairs_list {
        let path = path.as_ref();
        let new_name = new_name.as_ref();
        'retry: loop {
            let rename_res = do_rename(path, new_name, dry_run);
            match (rename_res, err_mode) {
                (Ok(_), _) => {
                    trace!("Rename from {path:?} to {new_name} successful.");
                    success_count += 1;
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Ignore) => {
                    debug!("Failed to rename {path:?} to {new_name}: {err}, ignoring.");
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Warn) => {
                    debug!("Failed to rename {path:?} to {new_name}: {err}. Prompting.");
                    println!(
                        "Failed to rename {:?} to {}: {err}",
                        Colour::Red.paint(format!("{path:?}")),
                        Colour::Red.paint(new_name),
                    );
                    let user_response = error_prompt("What to do with this file?", Some(OnErrorResponse::Skip))?;
                    trace!("User selected \"{user_response}\"");

                    match user_response {
                        OnErrorResponse::Skip => break 'retry,
                        OnErrorResponse::Retry => continue 'retry,
                        OnErrorResponse::Halt => Err(RenameError::UserHalt)?,
                    }
                }
                (Err(err), ErrorHandlingMode::Halt) => {
                    debug!("Failed to rename {path:?} to {new_name}: {err}. Halting.");
                    Err(err)?;
                }
            }
        }
    }

    info!("Successfully renamed {success_count} files");
    Ok(success_count)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BatchConfirmResponse {
    Proceed,
    Skip,
    Halt,
}
impl FromStr for BatchConfirmResponse {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "p" | "proceed" => Self::Proceed,
            "s" | "skip" => Self::Skip,
            "h" | "halt" => Self::Halt,
            other => Err(format!("\"{other}\" is not a valid response"))?,
        })
    }
}
impl fmt::Display for BatchConfirmResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::Proceed => "proceed",
            Self::Skip => "skip",
            Self::Halt => "halt",
        };
        write!(f, "{repr}")
    }
}

fn rename_files_confirm(
    pairs_list: &[(impl AsRef<Path>, impl AsRef<str>)],
    dry_run: bool,
    batch_size: usize,
    err_mode: ErrorHandlingMode,
) -> Result<usize, RenameError> {
    let mut success_count = 0;

    debug!("Renaming files with confirmation and batch size of {batch_size}.");
    let batch_count = ((pairs_list.len() as f64) / (batch_size as f64)).ceil() as usize;
    'batch: for (batch_idx, batch) in pairs_list.chunks(batch_size).enumerate() {
        trace!("Processing batch {batch_idx}.");

        // confirm batch
        println!(
            "Batch {}/{}{}:",
            Colour::Yellow.paint(format!("#{}", batch_idx + 1)),
            Colour::Green.paint(batch_count.to_string()),
            if dry_run {
                format!(" ({})", Colour::Red.paint("DRY RUN"))
            } else {
                "".into()
            }
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
        println!("{batch_info_text}");

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
        trace!("User selected \"{user_response}\"");

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
                let rename_res = do_rename(path, new_name, dry_run);
                match (rename_res, err_mode) {
                    (Ok(_), _) => {
                        trace!("Rename from {path:?} to {new_name} successful.");
                        success_count += 1;
                        break 'retry;
                    }
                    (Err(err), ErrorHandlingMode::Ignore) => {
                        debug!("Failed to rename {path:?} to {new_name}: {err}, ignoring.");
                        break 'retry;
                    }
                    (Err(err), ErrorHandlingMode::Warn) => {
                        debug!("Failed to rename {path:?} to {new_name}: {err}. Prompting.");
                        println!(
                            "Failed to rename {:?} to {}: {err}",
                            Colour::Red.paint(format!("{path:?}")),
                            Colour::Red.paint(new_name),
                        );
                        let user_response = error_prompt("What to do with this file?", Some(OnErrorResponse::Skip))?;
                        trace!("User selected \"{user_response}\"");

                        match user_response {
                            OnErrorResponse::Skip => break 'retry,
                            OnErrorResponse::Retry => continue 'retry,
                            OnErrorResponse::Halt => Err(RenameError::UserHalt)?,
                        }
                    }
                    (Err(err), ErrorHandlingMode::Halt) => {
                        debug!("Failed to rename {path:?} to {new_name}: {err}. Halting.");
                        Err(err)?;
                    }
                }
            }
        }
    }

    info!("Successfully renamed {success_count} files");
    Ok(success_count)
}

/// Perform rename on a single file.
fn do_rename(path: &Path, new_name: &str, dry_run: bool) -> io::Result<()> {
    trace!("Renaming {path:?} to {new_name}. Dry run: {dry_run}.");

    let new_abs_path = {
        let mut new_path = path
            .parent()
            .expect("paths should point to files at this point")
            .to_owned();
        new_path.push(new_name);
        new_path
    };

    if new_abs_path.try_exists()? {
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("renaming {path:?} to {new_abs_path:?} will overwrite an existing file"),
        ))?;
    }

    if dry_run {
        println!(
            "\tRename preview: {} -> {}",
            Colour::Yellow.paint(format!("{path:?}")),
            Colour::Green.paint(format!("{new_abs_path:?}")),
        );
    } else {
        trace!("New full path is {new_abs_path:?}");
        fs::rename(path, new_abs_path)?;
    }

    Ok(())
}
