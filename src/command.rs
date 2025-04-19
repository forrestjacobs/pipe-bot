use crate::tokenizer::Tokenizer;
use anyhow::{Context, Result, anyhow, bail};
use serenity::all::Context as SerenityContext;
use serenity::all::{ActivityData, ActivityType, ChannelId};

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
    pub async fn run(self, ctx: &SerenityContext) -> Result<()> {
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

    pub async fn handle(value: &str, ctx: &SerenityContext) -> Result<()> {
        Command::try_from(value)?.run(&ctx).await
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

        match tokenizer.next() {
            Some("message") => Ok(Command::Message {
                channel_id: tokenizer
                    .next()
                    .ok_or_else(|| anyhow!("expected channel ID"))?
                    .parse()
                    .context("channel ID must be a number")?,
                content: tokenizer.expect_rest().context("expected message")?,
            }),
            Some("playing") => parse_status(ActivityType::Playing, tokenizer),
            Some("listening_to") => parse_status(ActivityType::Listening, tokenizer),
            Some("watching") => parse_status(ActivityType::Watching, tokenizer),
            Some("competing_in") => parse_status(ActivityType::Competing, tokenizer),
            Some("clear_status") => {
                tokenizer.expect_none()?;
                Ok(Command::ClearStatus)
            }
            Some(name) => bail!("unrecognized command {name}"),
            None => bail!("expected command"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unrecognized_command() {
        let command = Command::try_from("lorem\n");
        assert_eq!(
            command.unwrap_err().to_string(),
            "unrecognized command lorem"
        );
    }

    #[test]
    fn parse_message() {
        let command = Command::try_from("message 12345 lorem ipsum\n");
        assert_eq!(
            command.ok(),
            Some(Command::Message {
                channel_id: ChannelId::new(12345),
                content: "lorem ipsum"
            })
        );
    }

    #[test]
    fn parse_message_missing_channel() {
        let command = Command::try_from("message\n");
        assert_eq!(command.unwrap_err().to_string(), "expected channel ID");
    }

    #[test]
    fn parse_message_bad_channel() {
        let command = Command::try_from("message lorem\n");
        assert_eq!(
            command.unwrap_err().to_string(),
            "channel ID must be a number"
        );
    }

    #[test]
    fn parse_message_missing_message() {
        let command = Command::try_from("message 12345\n");
        assert_eq!(command.unwrap_err().to_string(), "expected message");
    }

    #[test]
    fn parse_clear_status() {
        let command = Command::try_from("clear_status\n");
        assert_eq!(command.ok(), Some(Command::ClearStatus));
    }

    #[test]
    fn parse_clear_status_with_args() {
        let command = Command::try_from("clear_status lorem ipsum\n");
        assert_eq!(command.unwrap_err().to_string(), "unexpected token");
    }

    #[test]
    fn parse_playing_status() {
        let command = Command::try_from("playing a guitar\n");
        assert_eq!(
            command.ok(),
            Some(Command::Status {
                name: "a guitar",
                kind: ActivityType::Playing
            })
        );
    }

    #[test]
    fn parse_playing_empty_status() {
        let command = Command::try_from("playing\n");
        assert_eq!(command.unwrap_err().to_string(), "expected token");
    }
}
