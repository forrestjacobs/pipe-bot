use crate::command::Command;
use anyhow::Result;
use serenity::all::{Context, EventHandler, Ready};
use serenity::async_trait;
use std::io;

async fn handle(command: Result<Command<'_>>, ctx: &Context) -> Result<()> {
    command?.run(ctx).await
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let stdin = io::stdin();
        let mut buffer = String::new();

        loop {
            let command;
            {
                command = Command::from_reader(&mut stdin.lock(), &mut buffer);
            }
            if let Err(e) = handle(command, &ctx).await {
                eprintln!("{e}")
            }
        }
    }
}
