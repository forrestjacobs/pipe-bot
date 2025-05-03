use anyhow::bail;
use std::{error, fmt};

pub struct Tokenizer<'a> {
    original: &'a str,
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn map<V, F>(&self, value: &'a str, f: F) -> Result<V, TokenizerError>
    where
        F: FnOnce(&'a str) -> anyhow::Result<V>,
    {
        f(value).map_err(|e| {
            let index = value.as_ptr() as usize - self.original.as_ptr() as usize;
            TokenizerError {
                prefix: self.original[..index].to_string(),
                value: value.to_string(),
                suffix: self.original[(index + value.len())..].to_string(),
                wrapped: e,
            }
        })
    }

    pub fn next<V, F>(&mut self, f: F) -> Result<V, TokenizerError>
    where
        F: FnOnce(&'a str) -> anyhow::Result<V>,
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

    pub fn rest<V, F>(self, f: F) -> Result<V, TokenizerError>
    where
        F: FnOnce(&'a str) -> anyhow::Result<V>,
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
pub struct TokenizerError {
    prefix: String,
    value: String,
    suffix: String,
    wrapped: anyhow::Error,
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "| {}{}{}\n| {:>prefix_len$}{:^<value_len$} {}",
            self.prefix,
            self.value,
            self.suffix,
            "",
            "",
            self.wrapped,
            prefix_len = self.prefix.len()
                + (if !self.prefix.is_empty() && self.value.is_empty() && self.suffix.is_empty() {
                    1
                } else {
                    0
                }),
            value_len = if self.value.is_empty() {
                1
            } else {
                self.value.len()
            },
        )
    }
}

impl error::Error for TokenizerError {}

pub fn nonempty_str(description: &str) -> impl FnOnce(&str) -> anyhow::Result<&str> {
    return move |str| {
        if str.is_empty() {
            bail!("expected {}", description);
        }
        Ok(str)
    };
}

pub fn empty_str(str: &str) -> anyhow::Result<()> {
    if !str.is_empty() {
        bail!("unexpected text")
    }
    Ok(())
}
