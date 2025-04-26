use crate::tokenizer::Tokenizer;
use anyhow::{Context as AnyhowContext, Result, bail};
use mockall::automock;
use serenity::all::{ActivityData, ActivityType, ChannelId, Context};
use std::io::BufRead;

#[automock]
pub trait DiscordContext {
    async fn say(&self, channel_id: ChannelId, content: &str) -> serenity::Result<()>;
    fn set_activity(&self, activity: Option<ActivityData>);
}

impl DiscordContext for Context {
    async fn say(&self, channel_id: ChannelId, content: &str) -> serenity::Result<()> {
        channel_id.say(&self.http, content).await?;
        Ok(())
    }
    fn set_activity(&self, activity: Option<ActivityData>) {
        self.set_activity(activity);
    }
}

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

    pub async fn run<C: DiscordContext>(self, ctx: &C) -> Result<()> {
        match self {
            Command::Message {
                channel_id,
                content,
            } => {
                ctx.say(channel_id, content).await?;
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
    use mockall::predicate::*;
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

    #[tokio::test]
    async fn send_message() -> Result<()> {
        let channel_id = ChannelId::new(12345);
        let content = "Lorem ipsum";

        let mut ctx = MockDiscordContext::new();
        ctx.expect_say()
            .with(eq(channel_id), eq(content))
            .once()
            .returning(|_, _| Ok(()));

        Command::Message {
            channel_id,
            content,
        }
        .run(&ctx)
        .await
    }

    #[tokio::test]
    async fn send_message_error() {
        let channel_id = ChannelId::new(12345);
        let content = "Lorem ipsum";

        let mut ctx = MockDiscordContext::new();
        ctx.expect_say()
            .with(eq(channel_id), eq(content))
            .once()
            .returning(|_, _| Err(serenity::Error::Other("test error")));

        assert_eq!(
            Command::Message {
                channel_id,
                content,
            }
            .run(&ctx)
            .await
            .unwrap_err()
            .to_string(),
            "test error"
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

    #[tokio::test]
    async fn clear_status() -> Result<()> {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_set_activity()
            .withf(|d| d.is_none())
            .once()
            .return_const(());
        Command::ClearStatus.run(&ctx).await
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

    #[tokio::test]
    async fn set_playing_status() -> Result<()> {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_set_activity()
            .withf(|d| match d {
                Some(ActivityData {
                    name,
                    kind: ActivityType::Playing,
                    state: None,
                    url: None,
                }) if name == "a guitar" => true,
                _ => false,
            })
            .once()
            .return_const(());

        Command::Status {
            kind: ActivityType::Playing,
            name: "a guitar",
        }
        .run(&ctx)
        .await
    }
}
