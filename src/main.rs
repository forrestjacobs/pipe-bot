mod command;
mod handler;
mod token;
mod tokenizer;

use anyhow::Result;
use handler::Handler;
use serenity::{Client, all::GatewayIntents};
use token::get_token;

#[tokio::main]
async fn main() -> Result<()> {
    Client::builder(get_token()?, GatewayIntents::empty())
        .event_handler(Handler)
        .await?
        .start()
        .await?;
    Ok(())
}
