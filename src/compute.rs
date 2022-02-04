use std::{fmt, iter, path::Path};

use itertools::Itertools;
use log::info;
use rand::Rng;

use crate::char_set::CharSet;

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
) -> Result<Vec<(&Path, String)>, NameGenerationError>
where
    P: AsRef<Path>,
{
    /// The ratio of files to naming space at which we switch from
    /// `generate_on_demand` to `generate_then_match`.
    const STRATEGY_RATIO_THRESHOLD: f64 = 0.1;

    let naming_spaces_size = chars.len() * length;
    if files.len() > naming_spaces_size {
        return Err(NameGenerationError::InsufficientNamingSpace {
            needs: files.len(),
            space: naming_spaces_size,
        });
    }

    if files.len() > FILE_COUNT_MAX {
        return Err(NameGenerationError::TooManyFiles { count: files.len() });
    }

    let files_space_ratio = (files.len() as f64) / (naming_spaces_size as f64);
    if files_space_ratio < STRATEGY_RATIO_THRESHOLD {
        generate_on_demand(files, chars, length)
    } else {
        generate_then_match(files, chars, length)
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
    for file in files.iter() {
        // loop until an unused name is found
        let name = loop {
            let mut name = String::new();
            // push random characters into name
            for _ in 0..length {
                name.push(chars[rng.gen_range(0..chars.len())]);
            }
            // check if name is unused
            if !name_map.iter().any(|(_, existing_name)| existing_name == &name) {
                break name;
            }
        };
        name_map.push((file.as_ref(), name));
    }

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

    let permutation_count = chars.len().checked_pow(length as u32);
    if !matches!(permutation_count, Some(0..=PERMUTATION_COUNT_MAX)) {
        return Err(NameGenerationError::TooManyPermutations {
            char_set: chars,
            length,
        });
    }

    // generate all possible names
    let mut candidates = iter::repeat(chars.get_char_set())
        .take(length)
        .multi_cartesian_product()
        .map(|char_seq| char_seq.into_iter().cloned().collect::<String>())
        .collect::<Vec<_>>();

    let mut rng = rand::thread_rng();

    let mut name_map = vec![];
    for file in files.iter() {
        // select random name for each file
        let name = candidates.swap_remove(rng.gen_range(0..candidates.len()));
        name_map.push((file.as_ref(), name));
    }

    Ok(name_map)
}
