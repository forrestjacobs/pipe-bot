use crate::command_reader::{CommandReader, LineReader};
use crate::discord_context::DiscordContext;
use anyhow::Result;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use tokio::sync::Mutex;

pub async fn handle<R: LineReader, C: DiscordContext>(
    reader: &mut CommandReader<R>,
    ctx: &C,
) -> Result<()> {
    reader.next().await?.run(ctx).await
}

pub struct Handler<R> {
    reader: Mutex<CommandReader<R>>,
}

impl<R: LineReader + Send> Handler<R> {
    pub fn new(inner: R) -> Self {
        Self {
            reader: Mutex::new(CommandReader::new(inner)),
        }
    }
}

#[async_trait]
impl<R: LineReader + Send> EventHandler for Handler<R> {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let mut reader = self
            .reader
            .try_lock()
            .expect("reader lock should be held exactly once");
        loop {
            if let Err(e) = handle(&mut reader, &ctx).await {
                eprintln!("{e}")
            }
        }
    }
}
