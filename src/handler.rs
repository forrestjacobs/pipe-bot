use std::{error, fmt};
use crate::command_reader::{CommandReader, LineReader, ReadError};
use crate::discord_context::DiscordContext;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum HandleError {
    Read(ReadError),
    Serenity(serenity::Error),
}

impl fmt::Display for HandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandleError::Read(e) => fmt::Display::fmt(&e, f),
            HandleError::Serenity(e) => fmt::Display::fmt(&e, f),
        }
    }
}

impl error::Error for HandleError {}

pub async fn handle<R: LineReader, C: DiscordContext>(
    reader: &mut CommandReader<R>,
    ctx: &C,
) -> Result<(), HandleError> {
    reader.next().await.map_err(HandleError::Read)?.run(ctx).await.map_err(HandleError::Serenity)
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
