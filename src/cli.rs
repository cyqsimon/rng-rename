use core::fmt;
use std::{num::ParseIntError, path::PathBuf, str::FromStr};

use clap::Parser;
use derivative::Derivative;

use crate::char_set::CustomCharSet;

#[derive(Derivative, Clone, Parser)]
#[derivative(Debug)]
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
    /// Set to 0 to confirm all at once (may be undesirable if you are processing a large
    /// number of files).
    #[clap(
        long = "confirm-batch",
        value_name = "SIZE",
        default_value = "10",
        parse(try_from_str = parse_batch_size)
    )]
    pub confirm_batch_size: usize,

    /// Preview what will happen without actually performing the rename.
    #[clap(short = 'd', long = "dry-run")]
    pub dry_run: bool,

    /// How to handle the original file extension?
    ///
    /// E.g. Original file name: `tarball.tar.xz`
    ///
    /// New extension: `keep_all` = `tar.xz`; `keep_last` = `xz`;
    /// `static` = `<STATIC_EXT>`; `discard` = None
    ///
    /// For mode `static`, the option `--static-ext` must also be specified.
    ///
    /// Use with caution!
    #[clap(
        short = 'x',
        long = "ext-mode",
        value_name = "MODE",
        possible_values = ["keep_all", "keep_last", "static", "discard"],
        default_value = "keep_last"
    )]
    pub extension_mode_selection: ExtensionModeSelection,

    /// The static file extension to use when `--ext-mode=static`, without the leading dot.
    ///
    /// Any character that's not filename-safe will be removed.
    #[clap(
        long = "static-ext",
        value_name = "EXT",
        allow_hyphen_values = true,
        required_if_eq("extension-mode-selection", "static")
    )]
    pub static_ext: Option<String>,

    /// How to handle errors?
    ///
    /// What to do when an error is encountered (e.g. file does not exist).
    ///
    /// `ignore` = "Ignore the error silently and continue"; `warn` = "Prompt the user";
    /// `halt` = "Fail fast and exit immediately"
    #[clap(
        short = 'e',
        long = "error-handling-mode",
        value_name = "MODE",
        possible_values = ["ignore", "warn", "halt"],
        default_value = "warn"
    )]
    pub error_handling_mode: ErrorHandlingMode,

    /// Do not use unless you know what you're doing.
    ///
    /// Force use a specific random name generation strategy. Useful flag for testing performance.
    #[clap(
        long = "force-generation-strategy",
        value_name = "STRAT",
        possible_values = ["on_demand", "match"]
    )]
    pub force_generation_strategy: Option<NameGenerationStrategy>,

    /// The number of random characters for each name.
    ///
    /// The number of randomly-generated characters to use for each name.
    /// This does not include the static prefix and suffix (if specified)
    /// or the file extension.
    ///
    /// If the character set & length combination does not have enough permutations
    /// to cover all input files, the program will take no actions and fail fast.
    #[clap(short = 'l', long = "length", value_name = "LEN", default_value = "8")]
    pub name_length: usize,

    /// Prefix each name with a static string.
    ///
    /// Any character that's not filename-safe will be removed.
    #[clap(long = "prefix", value_name = "PREFIX", allow_hyphen_values = true)]
    pub name_prefix: Option<String>,

    /// Suffix each name with a static string (before the extension).
    ///
    /// Any character that's not filename-safe will be removed.
    #[clap(long = "suffix", value_name = "SUFFIX", allow_hyphen_values = true)]
    pub name_suffix: Option<String>,

    /// What random characters to use?
    ///
    /// Set the character set to use for random characters. Use `--case` to set
    /// upper, lower, or mixed case, if applicable to the character set you chose.
    ///
    /// `base64` uses base64url encoding (`[A-Za-z0-9-_]`) to be filename-safe.
    ///
    /// For mode `custom`, the option `--custom-chars` must also be specified.
    #[clap(
        short = 's',
        long = "char-set",
        alias = "charset",
        value_name = "SET",
        possible_values = ["letters", "numbers", "alpha_numeric", "base16", "base64", "custom"],
        default_value = "base16"
    )]
    pub char_set_selection: CharSetSelection,

    /// The character set to use when `--char-set=custom`.
    ///
    /// E.g. `--custom-chars=ABCDabcd`
    ///
    /// Inclusion of any character that's not filename-safe will cause an error.
    #[clap(
        long = "custom-chars",
        value_name = "CHARS",
        allow_hyphen_values = true,
        required_if_eq("char-set-selection", "custom"),
        parse(try_from_str)
    )]
    pub custom_chars: Option<CustomCharSet>,

    /// Upper case, lower case, or mixed? (if applicable)
    ///
    /// Set the character case for the random characters, if applicable to your
    /// character set of choice. If not specified, lower case will be used
    /// by default where applicable. If the pair specified is invalid, the program
    /// will take no actions and fail fast.
    ///
    /// Support table: `letters` - `upper|lower|mixed`; `numbers` - N/A;
    /// `alpha_numeric` - `upper|lower|mixed`; `base16` - `upper|lower`; `base64` - N/A;
    /// `custom` - N/A.
    #[clap(
        long = "case",
        value_name = "CASE",
        possible_values = ["upper", "lower", "mixed"],
    )]
    pub case: Option<Casing>,

    /// Use verbose logging.
    ///
    /// Set the verbosity level of logging. This flag can be specified multiple times.
    ///
    /// None = "Warn"; Once = "Info"; Twice = "Debug"; Thrice = "Trace"

    #[clap(short = 'v', long = "verbose", parse(from_occurrences = parse_verbosity))]
    pub verbosity: log::Level,

    /// The files to rename.
    ///
    /// Note: if any of your files starts with a hyphen (`-`), it could be misinterpreted
    /// as a flag and prevent the program from running.
    ///
    /// If so, please put all your flags and options in front of the list of files, then
    /// separate them with 2 hyphens. For example:
    ///
    ///  - Instead of `rng-rename --length 5 -file-1 -file-2`
    ///  - Run `rng-rename --length 5 -- -file-1 -file-2`
    #[derivative(Debug(format_with = "debug_vec_omit"))]
    #[clap(required = true, value_name = "FILES", verbatim_doc_comment)]
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
pub enum ExtensionModeSelection {
    KeepAll,
    KeepLast,
    Static,
    Discard,
}
impl FromStr for ExtensionModeSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "keep_all" => Self::KeepAll,
            "keep_last" => Self::KeepLast,
            "static" => Self::Static,
            "discard" => Self::Discard,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorHandlingMode {
    Ignore,
    Warn,
    Halt,
}
impl FromStr for ErrorHandlingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ignore" => Self::Ignore,
            "warn" => Self::Warn,
            "halt" => Self::Halt,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NameGenerationStrategy {
    OnDemand,
    Match,
}
impl FromStr for NameGenerationStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "on_demand" => Self::OnDemand,
            "match" => Self::Match,
            _ => unreachable!("Invalid values should be caught by clap"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharSetSelection {
    Custom,
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
            "custom" => Self::Custom,
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

/// Map input of `0` to `MAX`, as specified in the docs.
fn parse_batch_size(s: &str) -> Result<usize, ParseIntError> {
    let raw: usize = s.parse()?;
    Ok(match raw {
        0 => usize::MAX,
        other => other,
    })
}

fn parse_verbosity(occurrences: u64) -> log::Level {
    use log::Level::*;
    match occurrences {
        0 => Warn,
        1 => Info,
        2 => Debug,
        3.. => Trace,
    }
}

fn debug_vec_omit(v: &Vec<impl fmt::Debug>, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use fmt::Debug;
    use log::LevelFilter::*;

    match log::max_level() {
        Off | Error | Warn | Info | Debug => write!(f, "/* omitted */"),
        Trace => v.fmt(f),
    }
}
