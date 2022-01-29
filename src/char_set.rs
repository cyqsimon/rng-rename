use std::{fmt, ops::Index};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharSet {
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
impl TryFrom<(CharSetSelection, Option<Casing>)> for CharSet {
    type Error = String;

    fn try_from(pair: (CharSetSelection, Option<Casing>)) -> Result<Self, Self::Error> {
        use Casing::*;
        use CharSetSelection::*;
        match pair {
            (Letters, None | Some(Lower)) => Ok(Self::LettersLower),
            (Letters, Some(Upper)) => Ok(Self::LettersUpper),
            (Letters, Some(Mixed)) => Ok(Self::LettersMixed),
            (Numbers, None) => Ok(Self::Numbers),
            (AlphaNumeric, None | Some(Lower)) => Ok(Self::AlphaNumericLower),
            (AlphaNumeric, Some(Upper)) => Ok(Self::AlphaNumericUpper),
            (AlphaNumeric, Some(Mixed)) => Ok(Self::AlphaNumericMixed),
            (Base16, None | Some(Lower)) => Ok(Self::Base16Lower),
            (Base16, Some(Upper)) => Ok(Self::Base16Upper),
            (Base64, None) => Ok(Self::Base64),
            (char_set, Some(case)) => Err(format!(
                "The pair {{char_set: {:?}, case: {:?}}} is invalid",
                char_set, case
            )),
        }
    }
}
impl fmt::Display for CharSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CharSet::*;
        let repr = match self {
            LettersLower => "[a-z]",
            LettersUpper => "[A-Z]",
            LettersMixed => "[a-zA-Z]",
            Numbers => "[0-9]",
            AlphaNumericLower => "[a-z0-9]",
            AlphaNumericUpper => "[A-Z0-9]",
            AlphaNumericMixed => "[a-zA-Z0-9]",
            Base16Lower => "[0-9a-f]",
            Base16Upper => "[0-9A-F]",
            Base64 => "[A-Za-z0-9-_]",
        };
        write!(f, "{}", repr)
    }
}
impl Index<usize> for CharSet {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.get_char_set()[index]
    }
}
impl CharSet {
    pub fn get_char_set(&self) -> &[char] {
        use CharSet::*;
        match self {
            LettersLower => &LETTERS_L,
            LettersUpper => &LETTERS_U,
            LettersMixed => &LETTERS_M,
            Numbers => &NUMBERS,
            AlphaNumericLower => &ALPHA_NUMERIC_L,
            AlphaNumericUpper => &ALPHA_NUMERIC_U,
            AlphaNumericMixed => &ALPHA_NUMERIC_M,
            Base16Lower => &BASE_16_L,
            Base16Upper => &BASE_16_U,
            Base64 => &BASE_64,
        }
    }
    pub fn len(&self) -> usize {
        self.get_char_set().len()
    }
}
