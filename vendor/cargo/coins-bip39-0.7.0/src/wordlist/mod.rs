pub mod english;
pub use self::english::*;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
/// The error type returned while interacting with wordists.
pub enum WordlistError {
    /// Describes the error when the wordlist is queried at an invalid index.
    #[error("the index `{0}` is invalid")]
    InvalidIndex(usize),
    /// Describes the error when the wordlist does not contain the queried word.
    #[error("the word `{0}` is invalid")]
    InvalidWord(String),
}

// The Wordlist trait that every language's wordlist must implement.
pub trait Wordlist {
    /// The wordlist in original form.
    const WORDLIST: &'static str;

    /// Returns the word of a given index from the word list.
    fn get(index: usize) -> Result<String, WordlistError> {
        if index >= 2048 {
            return Err(WordlistError::InvalidIndex(index));
        }
        Ok(Self::get_all()[index].into())
    }

    /// Returns the index of a given word from the word list.
    fn get_index(word: &str) -> Result<usize, WordlistError> {
        match Self::get_all().iter().position(|element| element == &word) {
            Some(index) => Ok(index),
            None => Err(WordlistError::InvalidWord(word.into())),
        }
    }

    /// Returns the word list as a string.
    fn get_all() -> Vec<&'static str> {
        Self::WORDLIST.lines().collect::<Vec<&str>>()
    }
}
