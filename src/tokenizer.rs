use std::{error, fmt, ops::Range};

pub struct Tokenizer<'a> {
    original: &'a str,
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn map<V, E, F>(&self, value: &'a str, f: F) -> Result<V, TokenizerError<E>>
    where
        F: FnOnce(&'a str) -> Result<V, E>,
    {
        f(value).map_err(|e| {
            let start = value.as_ptr() as usize - self.original.as_ptr() as usize;
            TokenizerError {
                text: self.original.to_string(),
                range: start..(start + value.len()),
                wrapped: e,
            }
        })
    }

    pub fn next<V, E, F>(&mut self, f: F) -> Result<V, TokenizerError<E>>
    where
        F: FnOnce(&'a str) -> Result<V, E>,
    {
        let token = if let Some((token, rest)) = self.rest.split_once(char::is_whitespace) {
            self.rest = rest.trim_start();
            token
        } else {
            let token = self.rest;
            self.rest = &self.rest[self.rest.len()..];
            token
        };

        self.map(token, f)
    }

    pub fn rest<V, E, F>(self, f: F) -> Result<V, TokenizerError<E>>
    where
        F: FnOnce(&'a str) -> Result<V, E>,
    {
        self.map(self.rest, f)
    }
}

impl<'a> From<&'a str> for Tokenizer<'a> {
    fn from(value: &'a str) -> Self {
        Tokenizer {
            original: value,
            rest: value.trim(),
        }
    }
}

#[derive(Debug)]
pub struct TokenizerError<E> {
    text: String,
    range: Range<usize>,
    wrapped: E,
}

impl<E> TokenizerError<E> {
    pub fn map<F>(self, f: impl FnOnce(E) -> F) -> TokenizerError<F> {
        TokenizerError {
            text: self.text,
            range: self.range,
            wrapped: f(self.wrapped),
        }
    }
}

impl<E: fmt::Display> fmt::Display for TokenizerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut start = self.range.start;
        if start != 0 && start == self.text.len() {
            start += 1;
        }
        write!(
            f,
            "| {}\n| {}{} {}",
            self.text,
            " ".repeat(start),
            "^".repeat(self.range.len().max(1)),
            self.wrapped,
        )
    }
}

impl<E: fmt::Display + fmt::Debug> error::Error for TokenizerError<E> {}

pub fn nonempty_str(str: &str) -> Result<&str, ()> {
    if str.is_empty() { Err(()) } else { Ok(str) }
}

pub fn empty_str(str: &str) -> Result<(), ()> {
    if !str.is_empty() { Err(()) } else { Ok(()) }
}
