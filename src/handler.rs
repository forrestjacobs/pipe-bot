use crate::command_reader::{CommandReader, ReadError};
use crate::discord_context::DiscordContext;
use log::warn;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use std::{error, fmt};
use tokio::io::{AsyncRead, BufReader};
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum MainLoopError {
    Read(ReadError),
    Serenity(serenity::Error),
}

impl fmt::Display for MainLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainLoopError::Read(e) => write!(f, "{e}"),
            MainLoopError::Serenity(e) => write!(f, "Unable to run command: {e}"),
        }
    }
}

impl error::Error for MainLoopError {}

pub struct MainLoop<R> {
    reader: CommandReader<BufReader<R>>,
}

impl<R: AsyncRead + Unpin> MainLoop<R> {
    pub fn new(inner: R) -> Self {
        Self {
            reader: CommandReader::new(inner),
        }
    }

    pub async fn handle_once<C: DiscordContext>(&mut self, ctx: &C) -> Result<(), MainLoopError> {
        self.reader
            .next()
            .await
            .map_err(MainLoopError::Read)?
            .run(ctx)
            .await
            .map_err(MainLoopError::Serenity)
    }

    pub async fn handle<C: DiscordContext>(&mut self, ctx: &C) {
        loop {
            if let Err(e) = self.handle_once(ctx).await {
                warn!("{e}")
            }
        }
    }
}

pub struct Handler<R> {
    inner: Mutex<MainLoop<R>>,
}

impl<R: AsyncRead + Unpin> Handler<R> {
    pub fn new(inner: MainLoop<R>) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }
}

#[async_trait]
impl<R: AsyncRead + Send + Unpin> EventHandler for Handler<R> {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let mut main_loop = self
            .inner
            .try_lock()
            .expect("Unable to start handling events");
        main_loop.handle(&ctx).await;
    }
}
