use std::{path::PathBuf, str::FromStr};

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
pub struct CliArgs {
    /// Confirm before rename?
    ///
    /// Whether to confirm with the user before the rename action is performed.
    ///
    /// `none` = "Skip confirmation"; `batch` = "Confirm several at a time";
    /// `each` = "Confirm each one individually"
    #[clap(
        short = 'c',
        long = "confirm",
        value_name = "MODE",
        possible_values = ["none", "batch", "each"],
        default_value = "batch"
    )]
    pub confirm_mode: ConfirmMode,

    /// How many files to confirm in a batch? 0 = unlimited.
    ///
    /// The number of files to confirm in a batch. Only effective when `confirm = batch`.
    /// Set to 0 to confirm all at once (be careful if you are processing a large
    /// number of files).
    #[clap(long = "confirm-batch", value_name = "SIZE", default_value = "10")]
    pub confirm_batch_size: usize,

    /// Discard the file extension.
    ///
    /// Do not append the original file extensions when performing the rename.
    /// Use with care!
    #[clap(short = 'e', long = "no-ext")]
    pub no_extension: bool,

    /// How to handle errors?
    ///
    /// What to do when an error is encountered (e.g. file does not exist).
    ///
    /// `ignore` = "Ignore the error silently and continue"; `warn` = "Prompt the user";
    /// `fail` = "Fail fast and exit immediately"
    #[clap(
        short = 'h',
        long = "error-handling-mode",
        value_name = "MODE",
        possible_values = ["ignore", "warn", "fail"],
        default_value = "warn"
    )]
    pub error_handling_mode: ErrorHandlingMode,

    /// The number of random characters for each name.
    ///
    /// The number of randomly-generated characters to use for each name.
    /// This does not include the static prefix (if specified with `--prefix`)
    /// or the file extension.
    ///
    /// If the character set & length combination does not have enough permutations
    /// to cover all input files, the program will take no actions and fail fast.
    #[clap(short = 'l', long = "length", value_name = "LEN", default_value = "8")]
    pub name_length: usize,

    /// Prefix each name with a static string.
    #[clap(short = 'p', long = "prefix", value_name = "PREFIX", default_value = "")]
    pub name_prefix: String,

    /// What random characters to use?
    ///
    /// Set the character set to use for random characters. Use `--case` to set
    /// upper, lower, or mixed case, if applicable to the character set you chose.
    ///
    /// `base64` uses base64url encoding (`[A-Za-z0-9-_]`) to be file-name safe.
    #[clap(
        short = 's',
        long = "char-set",
        value_name = "SET",
        possible_values = ["letters", "numbers", "alpha_numeric", "base16", "base64"],
        default_value = "base16"
    )]
    pub char_set_selection: CharSetSelection,

    /// Upper case, lower case, or mixed? (if applicable)
    ///
    /// Set the character case for the random characters, if applicable to your
    /// character set of choice. If not specified, lower case will be used
    /// by default where applicable. If the pair specified is invalid, the program
    /// will take no actions and fail fast.
    ///
    /// Support table: `letters` - `upper`, `lower`, `mixed`; `numbers` - N/A;
    /// `alpha_numeric` - `upper`, `lower`, `mixed`; `base16` - `upper`, `lower`;
    /// `base64` - N/A
    #[clap(
        long = "case",
        value_name = "CASE",
        possible_values = ["upper", "lower", "mixed"],
    )]
    pub case: Option<Casing>,

    /// The files to rename.
    #[clap(required = true, value_name = "FILES")]
    pub files: Vec<PathBuf>,
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
pub enum ErrorHandlingMode {
    Ignore,
    Warn,
    Fail,
}
impl FromStr for ErrorHandlingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ignore" => Self::Ignore,
            "warn" => Self::Warn,
            "fail" => Self::Fail,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharSetSelection {
    Letters,
    Numbers,
    AlphaNumeric,
    Base16,
    Base64,
}
impl FromStr for CharSetSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "letters" => Self::Letters,
            "numbers" => Self::Numbers,
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
