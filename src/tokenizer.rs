use anyhow::{Result, anyhow, bail};

pub struct Tokenizer<'a> {
    str: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn expect_none(self) -> Result<()> {
        let str = self.str;
        if !str.is_empty() {
            bail!("unexpected token")
        }
        Ok(())
    }

    pub fn expect_rest(self) -> Result<&'a str> {
        let str = self.str;
        if str.is_empty() {
            bail!("expected token")
        }
        Ok(str)
    }

    pub fn expect_next(&mut self) -> Result<&'a str> {
        self.next().ok_or_else(|| anyhow!("expected token"))
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
            self.str = rest.trim_start();
            Some(token)
        } else {
            self.str = "";
            Some(str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_empty_string() {
        let mut tokenizer = Tokenizer::from("");
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn tokenize_one_word_string() {
        let tokenizer = Tokenizer::from("lorem");
        assert_eq!(Vec::from_iter(tokenizer), vec!["lorem"]);
    }

    #[test]
    fn tokenize_two_word_string() {
        let tokenizer = Tokenizer::from("lorem ipsum");
        assert_eq!(Vec::from_iter(tokenizer), vec!["lorem", "ipsum"]);
    }

    #[test]
    fn tokenize_whitespace_string() {
        let tokenizer = Tokenizer::from("  lorem  ipsum\tdolor\nsit\r\namet  ");
        assert_eq!(
            Vec::from_iter(tokenizer),
            vec!["lorem", "ipsum", "dolor", "sit", "amet"]
        );
    }

    #[test]
    fn expect_none() {
        let tokenizer = Tokenizer::from("");
        assert!(tokenizer.expect_none().is_ok());
    }

    #[test]
    fn expect_none_after_iterating() {
        let mut tokenizer = Tokenizer::from("lorem");
        tokenizer.next();
        assert!(tokenizer.expect_none().is_ok());
    }

    #[test]
    fn expect_none_with_token() {
        let tokenizer = Tokenizer::from("lorem ipsum");
        assert_eq!(
            tokenizer.expect_none().unwrap_err().to_string(),
            "unexpected token"
        );
    }

    #[test]
    fn expect_rest() {
        let tokenizer = Tokenizer::from("lorem ipsum");
        assert_eq!(tokenizer.expect_rest().unwrap(), "lorem ipsum");
    }

    #[test]
    fn expect_rest_after_iterating() {
        let mut tokenizer = Tokenizer::from("lorem ipsum dolor");
        tokenizer.next();
        assert_eq!(tokenizer.expect_rest().unwrap(), "ipsum dolor");
    }

    #[test]
    fn expect_next() {
        let mut tokenizer = Tokenizer::from("lorem");
        assert_eq!(tokenizer.expect_next().unwrap(), "lorem");
    }

    #[test]
    fn expect_next_without_token() {
        let mut tokenizer = Tokenizer::from("");
        assert_eq!(
            tokenizer.expect_next().unwrap_err().to_string(),
            "expected token"
        );
    }
}
