use crate::command::Command;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use std::io;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let stdin = io::stdin();
        let mut buffer = String::new();

        loop {
            buffer.clear();
            match stdin.read_line(&mut buffer) {
                Ok(0) => {}
                Ok(_) => {
                    if let Err(e) = Command::handle(buffer.as_str(), &ctx).await {
                        eprintln!("Error handling command: {e}")
                    }
                }
                Err(e) => eprintln!("Error reading line: {e}"),
            }
        }
    }
}
