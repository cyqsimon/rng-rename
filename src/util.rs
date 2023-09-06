use std::{fmt, io, str::FromStr};

use ansi_term::Colour;
use dialoguer::Input;

use crate::cli::ExtensionModeSelection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionMode {
    KeepAll,
    KeepLast,
    Static(String),
    Discard,
}
impl TryFrom<(ExtensionModeSelection, Option<String>)> for ExtensionMode {
    type Error = String;

    /// Convert from `ExtensionModeSelection` to `ExtensionMode`, optionally supplying a string
    /// to use for the static extension.
    fn try_from(value: (ExtensionModeSelection, Option<String>)) -> Result<Self, Self::Error> {
        use ExtensionModeSelection as S;
        Ok(match value {
            (S::KeepAll, _) => Self::KeepAll,
            (S::KeepLast, _) => Self::KeepLast,
            (S::Static, Some(ext)) => Self::Static(ext),
            (S::Static, None) => Err("`--static-ext` should be required by clap".to_string())?,
            (S::Discard, _) => Self::Discard,
        })
    }
}
impl fmt::Display for ExtensionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ExtensionMode as M;
        let repr = match self {
            M::KeepAll => "KeepAll".into(),
            M::KeepLast => "KeepLast".into(),
            M::Static(ext) => format!("Static(\"{}\")", ext),
            M::Discard => "Discard".into(),
        };
        write!(f, "{}", repr)
    }
}

/// Legal responses from the user when we encounter an error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OnErrorResponse {
    Skip,
    Retry,
    Halt,
}
impl FromStr for OnErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use OnErrorResponse as E;
        Ok(match s.to_lowercase().as_str() {
            "s" | "skip" => E::Skip,
            "r" | "retry" => E::Retry,
            "h" | "halt" => E::Halt,
            other => Err(format!("\"{}\" is not a valid response", other))?,
        })
    }
}
impl fmt::Display for OnErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OnErrorResponse as E;
        let repr = match self {
            E::Skip => "skip",
            E::Retry => "retry",
            E::Halt => "halt",
        };
        write!(f, "{}", repr)
    }
}

/// Prompt the user to produce an `OnErrorResponse`.
pub fn error_prompt<S>(question: S, default: Option<OnErrorResponse>) -> io::Result<OnErrorResponse>
where
    S: Into<String>,
{
    use Colour::Green;
    let prompt_text = format!(
        "{} You can {}({}), {}({}), or {}({})",
        question.into(),
        Green.paint("skip"),
        Green.paint("s"),
        Green.paint("retry"),
        Green.paint("r"),
        Green.paint("halt"),
        Green.paint("h")
    );

    let mut response = Input::new();
    if let Some(val) = default {
        response.default(val);
    }
    response.with_prompt(prompt_text).interact()
}
