use crate::command::Command;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncRead, BufReader, stdin};

pub struct Handler {
    pub file_path: Option<String>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        match &self.file_path {
            None => main_loop(stdin(), ctx).await,
            Some(path) => match File::open(path).await {
                Err(e) => {
                    eprintln!("{e}");
                    return;
                }
                Ok(file) => main_loop(file, ctx).await,
            },
        }
    }
}

async fn main_loop<R: AsyncRead + Unpin>(readable: R, ctx: Context) {
    let mut buffer = String::new();
    let mut reader = BufReader::new(readable);
    loop {
        if let Err(e) = Command::handle(&mut buffer, &mut reader, &ctx).await {
            eprintln!("{e}")
        }
    }
}
