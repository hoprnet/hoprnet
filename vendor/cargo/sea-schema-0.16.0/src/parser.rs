#![allow(dead_code)]

use sea_query::{Token, Tokenizer};

pub struct Parser {
    pub tokens: Tokenizer,
    pub curr: Option<Token>,
    pub last: Option<Token>,
}

impl Parser {
    pub fn new(string: &str) -> Self {
        Self {
            tokens: Tokenizer::new(string),
            curr: None,
            last: None,
        }
    }

    pub fn curr(&mut self) -> Option<&Token> {
        if self.curr.is_some() {
            self.curr.as_ref()
        } else {
            self.next()
        }
    }

    pub fn last(&mut self) -> Option<&Token> {
        self.last.as_ref()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&Token> {
        if self.curr.is_some() {
            self.last = std::mem::take(&mut self.curr);
        }

        if let Some(tok) = self.tokens.next() {
            if tok.is_space() {
                if let Some(tok) = self.tokens.next() {
                    self.curr = Some(tok);
                }
            } else {
                self.curr = Some(tok);
            }
        }
        self.curr.as_ref()
    }

    pub fn next_if_unquoted(&mut self, word: &str) -> bool {
        if let Some(tok) = self.curr() {
            if tok.is_unquoted() && tok.as_str().to_lowercase() == word.to_lowercase() {
                self.next();
                return true;
            }
        }
        false
    }

    pub fn next_if_quoted_any(&mut self) -> Option<&Token> {
        if let Some(tok) = self.curr() {
            if tok.is_quoted() {
                self.next();
                return self.last();
            }
        }
        None
    }

    pub fn next_if_unquoted_any(&mut self) -> Option<&Token> {
        if let Some(tok) = self.curr() {
            if tok.is_unquoted() {
                self.next();
                return self.last();
            }
        }
        None
    }

    pub fn next_if_punctuation(&mut self, word: &str) -> bool {
        if let Some(tok) = self.curr() {
            if tok.is_punctuation() && tok.as_str() == word {
                self.next();
                return true;
            }
        }
        false
    }

    pub fn curr_is_unquoted(&mut self) -> bool {
        self.curr().is_some() && self.curr().unwrap().is_unquoted()
    }

    pub fn curr_as_str(&mut self) -> &str {
        self.curr().unwrap().as_str()
    }
}
