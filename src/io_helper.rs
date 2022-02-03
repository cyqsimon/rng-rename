use std::{
    fmt, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use ansi_term::Colour;
use dialoguer::Input;
use log::{debug, trace};
use strum::EnumIter;

use crate::cli::ErrorHandlingMode;

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
                    let prompt_text = format!(
                        "\tWhat to do with this path? You can {}({}), {}({}), or {}({})",
                        Colour::Green.paint("skip"),
                        Colour::Green.paint("s"),
                        Colour::Green.paint("retry"),
                        Colour::Green.paint("r"),
                        Colour::Green.paint("halt"),
                        Colour::Green.paint("h")
                    );
                    let user_response = Input::new()
                        .with_prompt(prompt_text)
                        .default(OnErrorResponse::Skip)
                        .interact_text()?;
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
pub enum DedupError {
    IOError(io::Error),
    UserHalt,
}
impl fmt::Display for DedupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            DedupError::IOError(err) => err.to_string(),
            DedupError::UserHalt => "user halt".into(),
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
        DedupError::IOError(err)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
enum OnErrorResponse {
    Skip,
    Retry,
    Halt,
}
impl fmt::Display for OnErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OnErrorResponse::*;
        let repr = match self {
            Skip => "skip",
            Retry => "retry",
            Halt => "halt",
        };
        write!(f, "{}", repr)
    }
}
impl FromStr for OnErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "s" | "skip" => OnErrorResponse::Skip,
            "r" | "retry" => OnErrorResponse::Retry,
            "h" | "halt" => OnErrorResponse::Halt,
            other => Err(format!("\"{}\" is not a valid response", other))?,
        })
    }
}
