use std::{
    io,
    path::{Path, PathBuf},
};

use crate::cli::ErrorHandlingMode;

pub fn dedup_paths<P>(files: &[P], err_mode: ErrorHandlingMode) -> io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    todo!()
}
