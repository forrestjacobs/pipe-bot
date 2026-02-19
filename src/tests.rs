use crate::command_reader::{CommandReader, StdinReader};
use crate::{discord_context::MockDiscordContext, handler};
use indoc::indoc;
use mockall::predicate::*;
use serenity::all::{ActivityData, ActivityType, ChannelId};
use std::{collections::VecDeque, io::Cursor, pin::Pin, task};
use tokio::io::{AsyncRead, ReadBuf};

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

async fn handle<R: AsyncRead + Send + Unpin>(
    readable: R,
    ctx: &MockDiscordContext,
) -> anyhow::Result<()> {
    let mut reader = CommandReader::new(StdinReader::new(readable));
    handler::handle(&mut reader, ctx).await
}

async fn handle_bad_input<T: AsRef<[u8]> + Send + Unpin>(inner: T) -> String {
    let ctx = MockDiscordContext::new();
    handle(Cursor::new(inner), &ctx).await.unwrap_err().to_string()
}

#[tokio::test]
async fn parse_empty_command() {
    assert_eq!(
        handle_bad_input(b"\n").await,
        indoc! {"
            | 
            | ^ expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'"}
    );
}

#[tokio::test]
async fn parse_unrecognized_command() {
    assert_eq!(
        handle_bad_input(b"lorem\n").await,
        indoc! {"
            | lorem
            | ^^^^^ expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'"}
    );
}

#[tokio::test]
async fn parse_message_missing_channel() {
    assert_eq!(
        handle_bad_input(b"message\n").await,
        indoc! {"
            | message
            |         ^ expected channel ID"}
    );
}

#[tokio::test]
async fn parse_message_bad_channel() {
    assert_eq!(
        handle_bad_input(b"message lorem\n").await,
        indoc! {"
            | message lorem
            |         ^^^^^ expected channel ID"}
    );
}

#[tokio::test]
async fn parse_message_missing_message() {
    assert_eq!(
        handle_bad_input(b"message 12345\n").await,
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

    handle(Cursor::new(b"message 12345 lorem ipsum\n"), &ctx).await
}

#[tokio::test]
async fn send_message_error() {
    let mut ctx = MockDiscordContext::new();
    ctx.expect_say()
        .once()
        .returning(|_, _| Err(serenity::Error::Other("test error")));

    assert_eq!(
        handle(Cursor::new(b"message 12345 lorem ipsum\n"), &ctx)
            .await
            .unwrap_err()
            .to_string(),
        "test error"
    );
}

#[tokio::test]
async fn parse_clear_status_with_args() {
    assert_eq!(
        handle_bad_input(b"clear_status lorem ipsum\n").await,
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
    handle(Cursor::new(b"clear_status\n"), &ctx).await
}

#[tokio::test]
async fn parse_playing_empty_status() {
    assert_eq!(
        handle_bad_input(b"playing\n").await,
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

    handle(Cursor::new(b"playing a guitar\n"), &ctx).await
}

#[tokio::test]
async fn ignores_eofs() -> anyhow::Result<()> {
    let mut input = TestInput {
        lines: VecDeque::from(["".to_string(), "clear_status\n".to_string()]),
        calls: Vec::new(),
    };

    let mut ctx = MockDiscordContext::new();
    ctx.expect_set_activity()
        .withf(|d| d.is_none())
        .once()
        .return_const(());

    handle(&mut input, &ctx).await?;
    assert_eq!(input.calls, vec![0, 13]);
    Ok(())
}
