mod command;
mod config;
mod handler;
mod tokenizer;

use anyhow::Result;
use handler::Handler;
use serenity::{Client, all::GatewayIntents};
use config::get_config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = get_config()?;
    Client::builder(config.token, GatewayIntents::empty())
        .event_handler(Handler)
        .await?
        .start()
        .await?;
    Ok(())
}
