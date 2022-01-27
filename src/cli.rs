use std::{path::PathBuf, str::FromStr};

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CliArgs {
    /// Confirm before rename?
    #[clap(
        short = 'c',
        long = "confirm",
        value_name = "MODE",
        possible_values = ["none", "batch", "each"],
        default_value = "batch"
    )]
    confirm_mode: ConfirmMode,

    /// How many files to confirm in a batch? Set to 0 to confirm all at once.
    #[clap(long = "confirm-batch", value_name = "SIZE", default_value = "10")]
    confirm_batch_size: usize,

    /// Discard the file extension.
    #[clap(short = 'e', long = "no-extensions")]
    no_extension: bool,

    /// The number of random characters for each name.
    #[clap(short = 'l', long = "length", value_name = "LEN", default_value = "8")]
    name_length: usize,

    /// Prefix each name with a static string.
    #[clap(short = 'p', long = "prefix", value_name = "PREFIX")]
    name_prefix: String,

    /// What random characters to use?
    #[clap(
        short = 's',
        long = "char-set",
        value_name = "SET",
        possible_values = ["letters", "numbers", "alpha_numeric", "base64", "base64"],
        default_value = "base16"
    )]
    char_set: CharSet,

    /// Upper case, lower case, or mixed? (if applicable)
    #[clap(
        long = "case",
        value_name = "CASE",
        possible_values = ["upper", "lower", "mixed"],
        default_value = "lower"
    )]
    case: Casing,

    /// The files to rename.
    #[clap(required = true)]
    files: Vec<PathBuf>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConfirmMode {
    None,
    Batch,
    Each,
}
impl FromStr for ConfirmMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Self::None,
            "batch" => Self::Batch,
            "each" => Self::Each,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharSet {
    Letters,
    Numbers,
    AlphaNumeric,
    Base16,
    Base64,
}
impl FromStr for CharSet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "letters" => Self::Letters,
            "number" => Self::Numbers,
            "alpha_numeric" => Self::AlphaNumeric,
            "base16" => Self::Base16,
            "base64" => Self::Base64,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Casing {
    Upper,
    Lower,
    Mixed,
}
impl FromStr for Casing {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "upper" => Self::Upper,
            "lower" => Self::Lower,
            "mixed" => Self::Mixed,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}
