use std::{
    fmt, io, iter,
    path::{Path, PathBuf},
};

use ansi_term::Colour;
use itertools::Itertools;
use log::{debug, info, trace};
use rand::Rng;

use crate::{
    char_set::CharSet,
    cli::{ErrorHandlingMode, ExtensionMode, NameGenerationStrategy},
    util::{error_prompt, OnErrorResponse},
};

/// The hard-coded limit for the number of files that can be processed at once.
const FILE_COUNT_MAX: usize = 2usize.pow(24);
/// The hard-coded limit for the number of permutations that can be generated first.
const PERMUTATION_COUNT_MAX: usize = 2usize.pow(28);

#[derive(Debug, Clone)]
pub enum NameGenerationError {
    InsufficientNamingSpace { needs: usize, space: usize },
    TooManyFiles { count: usize },
    TooManyPermutations { char_set: CharSet, length: usize },
}
impl fmt::Display for NameGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NameGenerationError::*;
        let repr = match self {
            &InsufficientNamingSpace { needs, space } => {
                format!(
                    "This combination of character set and length cannot uniquely cover every file.\n\
                    There are {} files but only {} unique names available.",
                    needs, space
                )
            }
            &TooManyFiles { count } => {
                format!(
                    "Cannot process {} files at once. Currently the limit is {}.",
                    count, FILE_COUNT_MAX
                )
            }
            &TooManyPermutations { char_set, length } => {
                format!(
                    "Cannot enumerate all permutations with the character set {} and length {}.",
                    char_set, length
                )
            }
        };
        write!(f, "{}", repr)
    }
}
impl From<NameGenerationError> for String {
    fn from(err: NameGenerationError) -> Self {
        err.to_string()
    }
}

/// Generate random names and match them to each file.
pub fn generate_random_names<P>(
    files: &[P],
    chars: CharSet,
    length: usize,
    force_strategy: Option<NameGenerationStrategy>,
) -> Result<Vec<(&Path, String)>, NameGenerationError>
where
    P: AsRef<Path>,
{
    /// The ratio of files to naming space at which we switch from
    /// `generate_on_demand` to `generate_then_match`.
    const STRATEGY_RATIO_THRESHOLD: f64 = 0.1;

    trace!("Checking if there are enough permutations.");
    let naming_spaces_size = chars.len().pow(length as u32);
    if files.len() > naming_spaces_size {
        return Err(NameGenerationError::InsufficientNamingSpace {
            needs: files.len(),
            space: naming_spaces_size,
        });
    }

    trace!("Checking the number of files does not exceed the maximum.");
    if files.len() > FILE_COUNT_MAX {
        return Err(NameGenerationError::TooManyFiles { count: files.len() });
    }

    match force_strategy {
        Some(NameGenerationStrategy::OnDemand) => {
            debug!("Forcing \"generate on demand\" strategy.");
            generate_on_demand(files, chars, length)
        }
        Some(NameGenerationStrategy::Match) => {
            debug!("Forcing \"generate_then_match\" strategy.");
            generate_then_match(files, chars, length)
        }
        None => {
            let files_space_ratio = (files.len() as f64) / (naming_spaces_size as f64);
            trace!("Ratio of files to naming space is {:.2e}.", files_space_ratio);
            if files_space_ratio < STRATEGY_RATIO_THRESHOLD {
                generate_on_demand(files, chars, length)
            } else {
                generate_then_match(files, chars, length)
            }
        }
    }
}

/// Generate each random string independently. Potential conflicts
/// are resolved on demand by regenerating.
///
/// Use when the naming space is large and the files are few.
fn generate_on_demand<P>(
    files: &[P],
    chars: CharSet,
    length: usize,
) -> Result<Vec<(&Path, String)>, NameGenerationError>
where
    P: AsRef<Path>,
{
    info!("Using \"Generate on demand\" strategy.");

    let mut rng = rand::thread_rng();

    let mut name_map = vec![];
    trace!("Generating names for every file.");
    for file in files.iter() {
        // loop until an unused name is found
        let name = loop {
            let mut name = String::new();
            // push random characters into name
            for _ in 0..length {
                name.push(chars[rng.gen_range(0..chars.len())]);
            }
            // check if name is used
            if name_map.iter().any(|(_, existing_name)| existing_name == &name) {
                debug!("Random name conflict: \"{}\". Retrying.", name);
            } else {
                break name;
            }
        };
        name_map.push((file.as_ref(), name));
    }

    trace!("Generated {} random names.", files.len());
    Ok(name_map)
}

/// Generate all possible permutations first, then match them to files.
///
/// Use when the naming space is on the same order of magnitude as
/// the number of files.
fn generate_then_match<P>(
    files: &[P],
    chars: CharSet,
    length: usize,
) -> Result<Vec<(&Path, String)>, NameGenerationError>
where
    P: AsRef<Path>,
{
    info!("Using \"Generate then match\" strategy.");

    // check if the number of permutations is too large
    trace!("Checking if the number of permutations is too large.");
    let permutation_count = chars.len().checked_pow(length as u32);
    if !matches!(permutation_count, Some(0..=PERMUTATION_COUNT_MAX)) {
        return Err(NameGenerationError::TooManyPermutations {
            char_set: chars,
            length,
        });
    }

    // generate all possible names
    trace!("Generating all possible permutations.");
    let mut candidates = iter::repeat(chars.get_char_set())
        .take(length)
        .multi_cartesian_product()
        .map(|char_seq| char_seq.into_iter().cloned().collect::<String>())
        .collect::<Vec<_>>();

    let mut rng = rand::thread_rng();

    let mut name_map = vec![];
    trace!("Randomly matching files to generated names.");
    for file in files.iter() {
        // select random name for each file
        let name = candidates.swap_remove(rng.gen_range(0..candidates.len()));
        name_map.push((file.as_ref(), name));
    }

    debug!("Generated {} random names.", name_map.len());
    trace!("{:?}", name_map);
    Ok(name_map)
}

#[derive(Debug)]
pub enum NameFinaliseError {
    NotUtf8 { path: PathBuf },
    IOError(io::Error),
    UserHalt,
}
impl fmt::Display for NameFinaliseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NameFinaliseError::*;
        let repr = match self {
            NotUtf8 { path } => format!("{:?} is not UTF8", path),
            IOError(err) => err.to_string(),
            UserHalt => "user halt".into(),
        };
        write!(f, "{}", repr)
    }
}
impl From<NameFinaliseError> for String {
    fn from(err: NameFinaliseError) -> Self {
        err.to_string()
    }
}
impl From<io::Error> for NameFinaliseError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

/// Append the original file extension and the prefix to the new names.
///
/// The behaviour when an error is encountered depends on `err_mode`.
pub fn finalise_names<P, S>(
    file_random_name_pairs: Vec<(P, String)>,
    prefix: S,
    extension_mode: ExtensionMode,
    err_mode: ErrorHandlingMode,
) -> Result<Vec<(P, String)>, NameFinaliseError>
where
    P: AsRef<Path> + fmt::Debug,
    S: AsRef<str>,
{
    let mut final_pairs = vec![];

    // append original extension
    debug!("Appending extensions to generated file names.");
    for (path, mut random_name) in file_random_name_pairs {
        'retry: loop {
            let ext_res = match extension_mode {
                ExtensionMode::KeepAll => {
                    // TODO: awaiting implementation and stabilisation of `Path::file_suffix`
                    // afterwards this entire match block can be refactored
                    // see https://github.com/rust-lang/rust/issues/86319#issuecomment-996152668
                    path.as_ref()
                        .file_name()
                        .expect("paths should already be canonicalised")
                        .to_str()
                        .ok_or_else(|| NameFinaliseError::NotUtf8 {
                            path: path.as_ref().to_owned(),
                        })
                        .map(|mut name| {
                            // currently, the rules are:
                            // - `None`, if there is no file name;
                            // - `None`, if there is no embedded `.`;
                            // - `None`, if the file name begins with `.` and has no other `.`s within;
                            // - Otherwise, the portion of the file name starting with the first non-beginning `.`
                            if name.starts_with('.') {
                                name = &name[1..];
                            }
                            name.split_once('.').map(|(_, suffix)| suffix)
                        })
                }
                ExtensionMode::KeepLast => path
                    .as_ref()
                    .extension()
                    .map(|ext| {
                        ext.to_str().ok_or_else(|| NameFinaliseError::NotUtf8 {
                            path: path.as_ref().to_owned(),
                        })
                    })
                    .transpose(),
                ExtensionMode::Discard => Ok(None),
            };
            match (ext_res, err_mode) {
                (Ok(ext), _) => {
                    trace!("The new extension for {:?} is {:?}", path.as_ref(), ext);
                    let new_name_with_ext = match ext {
                        Some(ext) => {
                            random_name.push('.');
                            random_name.push_str(ext);
                            random_name
                        }
                        None => random_name,
                    };
                    final_pairs.push((path, new_name_with_ext));
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Ignore) => {
                    debug!("Error getting extension of {:?}: {}. Ignoring.", path.as_ref(), err);
                    break 'retry;
                }
                (Err(err), ErrorHandlingMode::Warn) => {
                    debug!("Error getting extension of {:?}: {}. Prompting.", path.as_ref(), err);
                    println!(
                        "Error getting extension of {}: {}",
                        Colour::Red.paint(format!("{:?}", path.as_ref())),
                        err
                    );
                    let user_response = error_prompt("What to do with this file?", Some(OnErrorResponse::Skip))?;
                    trace!("User selected \"{}\"", user_response);

                    match user_response {
                        OnErrorResponse::Skip => break 'retry,
                        OnErrorResponse::Retry => continue 'retry,
                        OnErrorResponse::Halt => Err(NameFinaliseError::UserHalt)?,
                    }
                }
                (Err(err), ErrorHandlingMode::Halt) => {
                    debug!("Error getting extension of {:?}: {}. Halting.", path.as_ref(), err);
                    Err(err)?;
                }
            }
        }
    }

    // append prefix
    trace!("Appending prefix to generated file names.");
    final_pairs
        .iter_mut()
        .for_each(|(_, name)| *name = format!("{}{}", prefix.as_ref(), name));

    debug!("Finalised names for {} files.", final_pairs.len());
    trace!("{:?}", final_pairs);
    Ok(final_pairs)
}
