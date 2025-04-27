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
            if let Err(e) = Command::handle(&mut buffer, &stdin, &ctx).await {
                eprintln!("{e}")
            }
        }
    }
}
