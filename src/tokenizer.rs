use anyhow::{Result, bail};

pub struct Tokenizer<'a> {
    str: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn expect_none(self) -> Result<()> {
        let str = self.str;
        if !str.is_empty() {
            bail!("unexpected arguments")
        }
        Ok(())
    }

    pub fn expect_rest(self) -> Result<&'a str> {
        let str = self.str;
        if str.is_empty() {
            bail!("expected string argument")
        }
        Ok(str)
    }
}

impl<'a> From<&'a str> for Tokenizer<'a> {
    fn from(value: &'a str) -> Self {
        Tokenizer { str: value.trim() }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let str = self.str;
        if str.len() == 0 {
            None
        } else if let Some((token, rest)) = str.split_once(char::is_whitespace) {
            self.str = rest;
            Some(token)
        } else {
            self.str = "";
            Some(str)
        }
    }
}
