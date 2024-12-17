use std::{collections::HashMap, error::Error, fmt, ops::Index, str::FromStr};

use itertools::Itertools;

use crate::cli::{Casing, CharSetSelection};

static LETTERS_L: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z',
];
static LETTERS_U: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
    'X', 'Y', 'Z',
];
static LETTERS_M: [char; 52] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z',
];
static NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
static ALPHA_NUMERIC_L: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
static ALPHA_NUMERIC_U: [char; 36] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
    'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
static ALPHA_NUMERIC_M: [char; 62] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
static BASE_16_L: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
static BASE_16_U: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];
static BASE_64: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
    'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_',
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseCustomCharSetError {
    IllegalChars(Vec<char>),
    DuplicateChars(Vec<char>),
}
impl Error for ParseCustomCharSetError {}
impl fmt::Display for ParseCustomCharSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn chars_to_string(chars: &[char]) -> String {
            chars.iter().map(|c| format!("\'{c}\'")).join(", ")
        }

        let repr = match self {
            Self::IllegalChars(chars) => {
                let chars = chars_to_string(chars);
                format!("the custom character set contains illegal characters: {chars}")
            }
            Self::DuplicateChars(chars) => {
                let chars = chars_to_string(chars);
                format!("the custom character set contains duplicate characters: {chars}")
            }
        };
        write!(f, "{repr}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCharSet {
    chars: Vec<char>,
}
impl FromStr for CustomCharSet {
    type Err = ParseCustomCharSetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use sanitize_filename as sf;
        use ParseCustomCharSetError as E;

        let illegal_chars: Vec<_> = s
            .chars()
            .filter(|c| {
                let c = c.to_string();
                c != sf::sanitize_with_options(
                    &c,
                    sf::Options {
                        windows: false, // this avoids filtering trailing dot
                        ..Default::default()
                    },
                )
            })
            .collect();
        if !illegal_chars.is_empty() {
            Err(E::IllegalChars(illegal_chars))?;
        }

        let duplicate_chars: Vec<_> = s
            .chars()
            .fold(HashMap::<char, usize>::new(), |mut map, c| {
                *map.entry(c).or_default() += 1;
                map
            })
            .into_iter()
            .filter_map(|(c, count)| (count > 1).then_some(c))
            .collect();
        if !duplicate_chars.is_empty() {
            Err(E::DuplicateChars(duplicate_chars))?;
        }

        let chars = s.chars().collect();
        Ok(Self { chars })
    }
}
impl fmt::Display for CustomCharSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chars = self.chars.iter().collect::<String>();
        write!(f, "{chars}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharSet {
    Custom(CustomCharSet),
    LettersLower,
    LettersUpper,
    LettersMixed,
    Numbers,
    AlphaNumericLower,
    AlphaNumericUpper,
    AlphaNumericMixed,
    Base16Lower,
    Base16Upper,
    Base64,
}
impl TryFrom<(CharSetSelection, Option<CustomCharSet>, Option<Casing>)> for CharSet {
    type Error = String;

    fn try_from(combination: (CharSetSelection, Option<CustomCharSet>, Option<Casing>)) -> Result<Self, Self::Error> {
        use Casing as C;
        use CharSetSelection as S;
        match combination {
            // when `--char-set=custom`, `--custom-chars` is guaranteed to be set
            (S::Custom, None, _) => unreachable!("`--custom-chars` should be required by clap"),
            // when `--custom-chars` is set, `--char-set=custom` must be true
            (S::Letters | S::Numbers | S::AlphaNumeric | S::Base16 | S::Base64, Some(_), _) => {
                Err("`--custom-chars` cannot be used unless `--char-set=custom`".to_string())
            }
            // valid combinations
            (S::Custom, Some(set), None) => Ok(Self::Custom(set)),
            (S::Letters, None, None | Some(C::Lower)) => Ok(Self::LettersLower),
            (S::Letters, None, Some(C::Upper)) => Ok(Self::LettersUpper),
            (S::Letters, None, Some(C::Mixed)) => Ok(Self::LettersMixed),
            (S::Numbers, None, None) => Ok(Self::Numbers),
            (S::AlphaNumeric, None, None | Some(C::Lower)) => Ok(Self::AlphaNumericLower),
            (S::AlphaNumeric, None, Some(C::Upper)) => Ok(Self::AlphaNumericUpper),
            (S::AlphaNumeric, None, Some(C::Mixed)) => Ok(Self::AlphaNumericMixed),
            (S::Base16, None, None | Some(C::Lower)) => Ok(Self::Base16Lower),
            (S::Base16, None, Some(C::Upper)) => Ok(Self::Base16Upper),
            (S::Base64, None, None) => Ok(Self::Base64),
            // incompatible `--char-set` and `--case` values
            (char_set_selection, _, Some(case)) => Err(format!(
                "the character set {char_set_selection:?} is incompatible with the case {case:?}"
            )),
        }
    }
}
impl Index<usize> for CharSet {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.get_char_set()[index]
    }
}
impl fmt::Display for CharSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::LettersLower => "[a-z]".into(),
            Self::LettersUpper => "[A-Z]".into(),
            Self::LettersMixed => "[a-zA-Z]".into(),
            Self::Numbers => "[0-9]".into(),
            Self::AlphaNumericLower => "[a-z0-9]".into(),
            Self::AlphaNumericUpper => "[A-Z0-9]".into(),
            Self::AlphaNumericMixed => "[a-zA-Z0-9]".into(),
            Self::Base16Lower => "[0-9a-f]".into(),
            Self::Base16Upper => "[0-9A-F]".into(),
            Self::Base64 => "[A-Za-z0-9-_]".into(),
            Self::Custom(chars) => format!("Custom(\"{chars}\")"),
        };
        write!(f, "{repr}")
    }
}
impl CharSet {
    pub fn get_char_set(&self) -> &[char] {
        match self {
            Self::LettersLower => &LETTERS_L,
            Self::LettersUpper => &LETTERS_U,
            Self::LettersMixed => &LETTERS_M,
            Self::Numbers => &NUMBERS,
            Self::AlphaNumericLower => &ALPHA_NUMERIC_L,
            Self::AlphaNumericUpper => &ALPHA_NUMERIC_U,
            Self::AlphaNumericMixed => &ALPHA_NUMERIC_M,
            Self::Base16Lower => &BASE_16_L,
            Self::Base16Upper => &BASE_16_U,
            Self::Base64 => &BASE_64,
            Self::Custom(set) => &set.chars,
        }
    }
    pub fn len(&self) -> usize {
        self.get_char_set().len()
    }
}
