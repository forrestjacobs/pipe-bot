use crate::tokenizer::{Tokenizer, TokenizerError, empty_str, nonempty_str};
use anyhow::{Context as AnyhowContext, bail};
use mockall::automock;
use serenity::all::{ActivityData, ActivityType, ChannelId, Context};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

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
    pub async fn handle<R: AsyncRead + Unpin, C: DiscordContext>(
        buffer: &mut String,
        reader: &mut BufReader<R>,
        ctx: &C,
    ) -> anyhow::Result<()> {
        buffer.clear();
        while reader.read_line(buffer).await? == 0 {}
        let command = Command::try_from(buffer.as_str().trim_end_matches('\n'))?;
        command.run(ctx).await
    }

    pub async fn run<C: DiscordContext>(self, ctx: &C) -> anyhow::Result<()> {
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

enum CommandName {
    Message,
    Status(ActivityType),
    ClearStatus,
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = TokenizerError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut tokenizer = Tokenizer::from(value);

        let name = tokenizer.next(|v| {
            Ok(match v {
                "message" => CommandName::Message,
                "playing" => CommandName::Status(ActivityType::Playing),
                "listening_to" => CommandName::Status(ActivityType::Listening),
                "watching" => CommandName::Status(ActivityType::Watching),
                "competing_in" => CommandName::Status(ActivityType::Competing),
                "clear_status" => CommandName::ClearStatus,
                _ => bail!("expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'"),
            })
        })?;

        Ok(match name {
            CommandName::Message => Command::Message {
                channel_id: tokenizer.next(|v| v.parse().context("expected channel ID"))?,
                content: tokenizer.rest(nonempty_str("message"))?,
            },
            CommandName::Status(kind) => Command::Status {
                kind,
                name: tokenizer.rest(nonempty_str("text"))?,
            },
            CommandName::ClearStatus => {
                tokenizer.rest(empty_str)?;
                Command::ClearStatus
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use mockall::predicate::*;
    use std::collections::VecDeque;
    use std::io::Cursor;
    use std::pin::Pin;
    use std::task;
    use tokio::io::ReadBuf;

    struct TestInput {
        lines: VecDeque<String>,
        calls: Vec<usize>,
    }

    impl AsyncRead for TestInput {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut task::Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> task::Poll<std::io::Result<()>> {
            if let Some(line) = self.lines.pop_front() {
                buf.put_slice(line.as_bytes());
                self.calls.push(line.len());
            }
            task::Poll::Ready(Ok(()))
        }
    }

    async fn handle_with_context<R: AsyncRead + Unpin>(
        readable: R,
        ctx: &MockDiscordContext,
    ) -> anyhow::Result<()> {
        let mut reader = BufReader::new(readable);
        Command::handle(&mut String::new(), &mut reader, ctx).await
    }

    async fn handle<R: AsyncRead + Unpin>(readable: R) -> anyhow::Result<()> {
        let ctx = MockDiscordContext::new();
        handle_with_context(readable, &ctx).await
    }

    #[tokio::test]
    async fn parse_empty_command() {
        assert_eq!(
            handle(Cursor::new(b"\n")).await.unwrap_err().to_string(),
            indoc! {"
                | 
                | ^ expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'"}
        );
    }

    #[tokio::test]
    async fn parse_unrecognized_command() {
        assert_eq!(
            handle(Cursor::new(b"lorem\n"))
                .await
                .unwrap_err()
                .to_string(),
            indoc! {"
                | lorem
                | ^^^^^ expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'"}
        );
    }

    #[tokio::test]
    async fn parse_message_missing_channel() {
        assert_eq!(
            handle(Cursor::new(b"message\n"))
                .await
                .unwrap_err()
                .to_string(),
            indoc! {"
                | message
                |         ^ expected channel ID"}
        );
    }

    #[tokio::test]
    async fn parse_message_bad_channel() {
        assert_eq!(
            handle(Cursor::new(b"message lorem\n"))
                .await
                .unwrap_err()
                .to_string(),
            indoc! {"
                | message lorem
                |         ^^^^^ expected channel ID"}
        );
    }

    #[tokio::test]
    async fn parse_message_missing_message() {
        assert_eq!(
            handle(Cursor::new(b"message 12345\n"))
                .await
                .unwrap_err()
                .to_string(),
            indoc! {"
                | message 12345
                |               ^ expected message"}
        );
    }

    #[tokio::test]
    async fn send_message() -> anyhow::Result<()> {
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
            indoc! {"
                | clear_status lorem ipsum
                |              ^^^^^^^^^^^ unexpected text"}
        );
    }

    #[tokio::test]
    async fn clear_status() -> anyhow::Result<()> {
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
            indoc! {"
                | playing
                |         ^ expected text"}
        );
    }

    #[tokio::test]
    async fn set_playing_status() -> anyhow::Result<()> {
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
    async fn ignores_eofs() -> anyhow::Result<()> {
        let mut stdin = TestInput {
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
