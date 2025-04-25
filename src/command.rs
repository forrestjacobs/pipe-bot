use crate::tokenizer::Tokenizer;
use anyhow::{Context as AnyhowContext, Result, bail};
use serenity::all::{ActivityData, ActivityType, ChannelId, Context};
use std::io::BufRead;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Message {
        channel_id: ChannelId,
        content: &'a str,
    },
    Status {
        name: &'a str,
        kind: ActivityType,
    },
    ClearStatus,
}

impl Command<'_> {
    pub fn from_reader<'a, B: BufRead>(
        reader: &mut B,
        buffer: &'a mut String,
    ) -> Result<Command<'a>> {
        buffer.clear();
        while reader.read_line(buffer)? == 0 {}
        Command::try_from(buffer.as_str())
    }

    pub async fn run(self, ctx: &Context) -> Result<()> {
        match self {
            Command::Message {
                channel_id,
                content,
            } => {
                channel_id.say(&ctx.http, content).await?;
            }
            Command::Status { name, kind } => {
                ctx.set_activity(Some(ActivityData {
                    name: name.into(),
                    kind,
                    state: None,
                    url: None,
                }));
            }
            Command::ClearStatus => {
                ctx.set_activity(None);
            }
        }
        Ok(())
    }
}

fn parse_status(kind: ActivityType, tokenizer: Tokenizer<'_>) -> Result<Command> {
    Ok(Command::Status {
        name: tokenizer.expect_rest()?,
        kind,
    })
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        let mut tokenizer = Tokenizer::from(value);

        match tokenizer.expect_next().context("expected command")? {
            "message" => Ok(Command::Message {
                channel_id: tokenizer
                    .expect_next()
                    .context("expected channel ID")?
                    .parse()
                    .context("channel ID must be a number")?,
                content: tokenizer.expect_rest().context("expected message")?,
            }),
            "playing" => parse_status(ActivityType::Playing, tokenizer),
            "listening_to" => parse_status(ActivityType::Listening, tokenizer),
            "watching" => parse_status(ActivityType::Watching, tokenizer),
            "competing_in" => parse_status(ActivityType::Competing, tokenizer),
            "clear_status" => {
                tokenizer.expect_none()?;
                Ok(Command::ClearStatus)
            }
            name => bail!("unrecognized command {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn parse<'a>(value: &[u8], buffer: &'a mut String) -> Result<Command<'a>> {
        let mut cursor = io::Cursor::new(value);
        Command::from_reader(&mut cursor, buffer)
    }

    #[test]
    fn parse_missing_command() {
        assert_eq!(
            parse(b"\n", &mut String::new()).unwrap_err().to_string(),
            "expected command"
        );
    }

    #[test]
    fn parse_unrecognized_command() {
        assert_eq!(
            parse(b"lorem\n", &mut String::new())
                .unwrap_err()
                .to_string(),
            "unrecognized command lorem"
        );
    }

    #[test]
    fn parse_message() {
        assert_eq!(
            parse(b"message 12345 lorem ipsum\n", &mut String::new()).unwrap(),
            Command::Message {
                channel_id: ChannelId::new(12345),
                content: "lorem ipsum"
            }
        );
    }

    #[test]
    fn parse_message_missing_channel() {
        assert_eq!(
            parse(b"message\n", &mut String::new())
                .unwrap_err()
                .to_string(),
            "expected channel ID"
        );
    }

    #[test]
    fn parse_message_bad_channel() {
        assert_eq!(
            parse(b"message lorem\n", &mut String::new())
                .unwrap_err()
                .to_string(),
            "channel ID must be a number"
        );
    }

    #[test]
    fn parse_message_missing_message() {
        assert_eq!(
            parse(b"message 12345\n", &mut String::new())
                .unwrap_err()
                .to_string(),
            "expected message"
        );
    }

    #[test]
    fn parse_clear_status() {
        assert_eq!(
            parse(b"clear_status\n", &mut String::new()).unwrap(),
            Command::ClearStatus
        );
    }

    #[test]
    fn parse_clear_status_with_args() {
        assert_eq!(
            parse(b"clear_status lorem ipsum\n", &mut String::new())
                .unwrap_err()
                .to_string(),
            "unexpected token"
        );
    }

    #[test]
    fn parse_playing_status() {
        assert_eq!(
            parse(b"playing a guitar\n", &mut String::new()).unwrap(),
            Command::Status {
                name: "a guitar",
                kind: ActivityType::Playing
            }
        );
    }

    #[test]
    fn parse_playing_empty_status() {
        assert_eq!(
            parse(b"playing\n", &mut String::new())
                .unwrap_err()
                .to_string(),
            "expected token"
        );
    }
}
