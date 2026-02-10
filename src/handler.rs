use crate::command::DiscordContext;
use crate::producer::Producer;
use anyhow::Result;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use tokio::io::AsyncRead;
use tokio::sync::RwLock;

pub async fn handle<R: AsyncRead + Unpin, C: DiscordContext>(
    producer: &mut Producer<R>,
    ctx: &C,
) -> Result<()> {
    producer.next().await?.run(ctx).await
}

pub struct Handler<R> {
    producer: RwLock<Producer<R>>,
}

impl<R: AsyncRead + Send + Sync + Unpin> Handler<R> {
    pub fn new(readable: R) -> Self {
        Self {
            producer: RwLock::new(Producer::new(readable)),
        }
    }
}

#[async_trait]
impl<R: AsyncRead + Send + Sync + Unpin> EventHandler for Handler<R> {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let mut producer = self.producer.write().await;
        loop {
            if let Err(e) = handle(&mut producer, &ctx).await {
                eprintln!("{e}")
            }
        }
    }
}
