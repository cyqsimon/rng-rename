use std::{fmt, io, str::FromStr};

use ansi_term::Colour;
use dialoguer::Input;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OnErrorResponse {
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
        use OnErrorResponse::*;
        Ok(match s.to_lowercase().as_str() {
            "s" | "skip" => Skip,
            "r" | "retry" => Retry,
            "h" | "halt" => Halt,
            other => Err(format!("\"{}\" is not a valid response", other))?,
        })
    }
}

pub fn error_prompt<S>(question: S, default: Option<OnErrorResponse>) -> io::Result<OnErrorResponse>
where
    S: Into<String>,
{
    use Colour::Green;
    let prompt_text = format!(
        "\t{} You can {}({}), {}({}), or {}({})",
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
