use crate::tokenizer::Tokenizer;
use anyhow::{Context as AnyhowContext, Result, bail};
use mockall::automock;
use serenity::all::{ActivityData, ActivityType, ChannelId, Context};
use std::io::{BufRead, Stdin, StdinLock};

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

pub trait Readable<Reader: BufRead> {
    fn reader(self) -> Reader;
}
impl Readable<StdinLock<'static>> for &Stdin {
    fn reader(self) -> StdinLock<'static> {
        self.lock()
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

    pub async fn handle<B: BufRead, R: Readable<B>, C: DiscordContext>(
        buffer: &mut String,
        readable: R,
        ctx: &C,
    ) -> Result<()> {
        let command = Command::from_reader(&mut readable.reader(), buffer);
        command?.run(ctx).await
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
    use std::collections::VecDeque;
    use std::io::{Cursor, Read};

    impl<R: BufRead> Readable<R> for R {
        fn reader(self) -> R {
            self
        }
    }

    struct TestStdin {
        lines: VecDeque<String>,
        calls: Vec<usize>,
    }

    impl Read for &mut TestStdin {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            unimplemented!()
        }
    }
    impl BufRead for &mut TestStdin {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            unimplemented!()
        }
        fn consume(&mut self, _: usize) {
            unimplemented!()
        }
        fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
            let line = self.lines.pop_front().unwrap();
            buf.push_str(&line);
            let len = line.len();
            self.calls.push(len);
            Ok(len)
        }
    }

    async fn handle_with_context<B: BufRead, R: Readable<B>>(
        readable: R,
        ctx: &MockDiscordContext,
    ) -> Result<()> {
        Command::handle(&mut String::new(), readable, ctx).await
    }

    async fn handle<B: BufRead, R: Readable<B>>(readable: R) -> Result<()> {
        let ctx = MockDiscordContext::new();
        handle_with_context(readable, &ctx).await
    }

    #[tokio::test]
    async fn parse_unrecognized_command() {
        assert_eq!(
            handle(Cursor::new(b"lorem\n"))
                .await
                .unwrap_err()
                .to_string(),
            "unrecognized command lorem"
        );
    }

    #[tokio::test]
    async fn parse_message_missing_channel() {
        assert_eq!(
            handle(Cursor::new(b"message\n"))
                .await
                .unwrap_err()
                .to_string(),
            "expected channel ID"
        );
    }

    #[tokio::test]
    async fn parse_message_bad_channel() {
        assert_eq!(
            handle(Cursor::new(b"message lorem\n"))
                .await
                .unwrap_err()
                .to_string(),
            "channel ID must be a number"
        );
    }

    #[tokio::test]
    async fn parse_message_missing_message() {
        assert_eq!(
            handle(Cursor::new(b"message 12345\n"))
                .await
                .unwrap_err()
                .to_string(),
            "expected message"
        );
    }

    #[tokio::test]
    async fn send_message() -> Result<()> {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_say()
            .with(eq(ChannelId::new(12345)), eq("lorem ipsum"))
            .once()
            .returning(|_, _| Ok(()));

        handle_with_context(Cursor::new(b"message 12345 lorem ipsum\n"), &ctx).await
    }

    #[tokio::test]
    async fn send_message_error() {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_say()
            .once()
            .returning(|_, _| Err(serenity::Error::Other("test error")));

        assert_eq!(
            handle_with_context(Cursor::new(b"message 12345 lorem ipsum\n"), &ctx)
                .await
                .unwrap_err()
                .to_string(),
            "test error"
        );
    }

    #[tokio::test]
    async fn parse_clear_status_with_args() {
        assert_eq!(
            handle(Cursor::new(b"clear_status lorem ipsum\n"),)
                .await
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
        handle_with_context(Cursor::new(b"clear_status\n"), &ctx).await
    }

    #[tokio::test]
    async fn parse_playing_empty_status() {
        assert_eq!(
            handle(Cursor::new(b"playing\n"))
                .await
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

        handle_with_context(Cursor::new(b"playing a guitar\n"), &ctx).await
    }

    #[tokio::test]
    async fn ignores_eofs() -> Result<()> {
        let mut stdin = TestStdin {
            lines: VecDeque::from(["".to_string(), "clear_status\n".to_string()]),
            calls: Vec::new(),
        };

        let mut ctx = MockDiscordContext::new();
        ctx.expect_set_activity()
            .withf(|d| d.is_none())
            .once()
            .return_const(());

        handle_with_context(&mut stdin, &ctx).await?;
        assert_eq!(stdin.calls, vec![0, 13]);
        Ok(())
    }
}
