use core::fmt;
use std::{num::ParseIntError, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use derivative::Derivative;

use crate::char_set::CustomCharSet;

#[derive(Derivative, Clone, Parser)]
#[derivative(Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    #[command(subcommand)]
    pub sub_cmd: Option<SubCmd>,

    /// Confirm before rename?
    ///
    /// Whether to confirm with the user before the rename action is performed.
    ///
    /// `none` = "Skip confirmation"; `batch` = "Confirm several at a time";
    /// `each` = "Confirm each one individually"
    #[arg(
        short = 'c',
        long = "confirm",
        value_name = "MODE",
        value_enum,
        default_value_t = ConfirmMode::Batch
    )]
    pub confirm_mode: ConfirmMode,

    /// How many files to confirm in a batch? 0 = unlimited.
    ///
    /// The number of files to confirm in a batch. Only effective when `confirm = batch`.
    /// Set to 0 to confirm all at once (may be undesirable if you are processing a large
    /// number of files).
    #[arg(
        long = "confirm-batch",
        value_name = "SIZE",
        default_value_t = 10,
        value_parser = parse_batch_size
    )]
    pub confirm_batch_size: usize,

    /// Preview what will happen without actually performing the rename.
    #[arg(short = 'd', long = "dry-run")]
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
    #[arg(
        short = 'x',
        long = "ext-mode",
        value_name = "MODE",
        value_enum,
        default_value_t = ExtensionModeSelection::KeepLast
    )]
    pub extension_mode_selection: ExtensionModeSelection,

    /// The static file extension to use when `--ext-mode=static`, without the leading dot.
    ///
    /// Any character that's not filename-safe will be removed.
    #[arg(
        long = "static-ext",
        value_name = "EXT",
        allow_hyphen_values = true,
        required_if_eq("extension_mode_selection", "static")
    )]
    pub static_ext: Option<String>,

    /// How to handle errors?
    ///
    /// What to do when an error is encountered (e.g. file does not exist).
    ///
    /// `ignore` = "Ignore the error silently and continue"; `warn` = "Prompt the user";
    /// `halt` = "Fail fast and exit immediately"
    #[arg(
        short = 'e',
        long = "error-handling-mode",
        value_name = "MODE",
        value_enum,
        default_value_t = ErrorHandlingMode::Warn
    )]
    pub error_handling_mode: ErrorHandlingMode,

    /// Do not use unless you know what you're doing.
    ///
    /// Force use a specific random name generation strategy. Useful flag for testing performance.
    #[arg(long = "force-generation-strategy", value_name = "STRAT", value_enum)]
    pub force_generation_strategy: Option<NameGenerationStrategy>,

    /// The number of random characters for each name.
    ///
    /// The number of randomly-generated characters to use for each name.
    /// This does not include the static prefix and suffix (if specified)
    /// or the file extension.
    ///
    /// If the character set & length combination does not have enough permutations
    /// to cover all input files, the program will take no actions and fail fast.
    #[arg(short = 'l', long = "length", value_name = "LEN", default_value = "8")]
    pub name_length: usize,

    /// Prefix each name with a static string.
    ///
    /// Any character that's not filename-safe will be removed.
    #[arg(long = "prefix", value_name = "PREFIX", allow_hyphen_values = true)]
    pub name_prefix: Option<String>,

    /// Suffix each name with a static string (before the extension).
    ///
    /// Any character that's not filename-safe will be removed.
    #[arg(long = "suffix", value_name = "SUFFIX", allow_hyphen_values = true)]
    pub name_suffix: Option<String>,

    /// What random characters to use?
    ///
    /// Set the character set to use for random characters. Use `--case` to set
    /// upper, lower, or mixed case, if applicable to the character set you chose.
    ///
    /// `base64` uses base64url encoding (`[A-Za-z0-9-_]`) to be filename-safe.
    ///
    /// For mode `custom`, the option `--custom-chars` must also be specified.
    #[arg(
        short = 's',
        long = "char-set",
        alias = "charset",
        value_name = "SET",
        value_enum,
        default_value_t = CharSetSelection::Base16
    )]
    pub char_set_selection: CharSetSelection,

    /// The character set to use when `--char-set=custom`.
    ///
    /// E.g. `--custom-chars=ABCDabcd`
    ///
    /// Inclusion of any character that's not filename-safe will cause an error.
    #[arg(
        long = "custom-chars",
        value_name = "CHARS",
        allow_hyphen_values = true,
        required_if_eq("char_set_selection", "custom")
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
    #[arg(long = "case", value_name = "CASE")]
    pub case: Option<Casing>,

    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    /// The files to rename.
    ///
    /// Note: if any of your files starts with a hyphen (`-`)
    /// or has the exact same name of one of the subcommands (e.g. `help`),
    /// it could be misinterpreted as a flag/subcommand and prevent the program from running.
    ///
    /// If so, please put all your flags and options in front of the list of files, then
    /// separate them with 2 hyphens. For example:
    ///
    ///  - Instead of `rng-rename --length 5 -file-1 -file-2`
    ///  - Run `rng-rename --length 5 -- -file-1 -file-2`
    #[derivative(Debug(format_with = "debug_vec_omit"))]
    #[arg(
        required = true,
        value_name = "FILES",
        value_hint(ValueHint::AnyPath),
        verbatim_doc_comment
    )]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
#[command(subcommand_negates_reqs(true))]
pub enum SubCmd {
    /// Generate a completion script for `rng-rename` to stdout.
    ///
    /// E.g. `rng-rename complete bash > ~/.local/share/bash-completion/completions/rng-rename`
    Complete {
        /// The type of shell.
        #[arg(required = true, value_name = "SHELL", value_enum)]
        shell_type: Shell,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ConfirmMode {
    None,
    Batch,
    Each,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ExtensionModeSelection {
    KeepAll,
    KeepLast,
    Static,
    Discard,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ErrorHandlingMode {
    Ignore,
    Warn,
    Halt,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum NameGenerationStrategy {
    OnDemand,
    Match,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum CharSetSelection {
    Custom,
    Letters,
    Numbers,
    AlphaNumeric,
    Base16,
    Base64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Casing {
    Upper,
    Lower,
    Mixed,
}

/// Map input of `0` to `MAX`, as specified in the docs.
fn parse_batch_size(s: &str) -> Result<usize, ParseIntError> {
    let raw: usize = s.parse()?;
    Ok(match raw {
        0 => usize::MAX,
        other => other,
    })
}

fn debug_vec_omit(v: &Vec<impl fmt::Debug>, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use fmt::Debug;
    use log::LevelFilter::*;

    match log::max_level() {
        Off | Error | Warn | Info | Debug => write!(f, "/* omitted */"),
        Trace => v.fmt(f),
    }
}
