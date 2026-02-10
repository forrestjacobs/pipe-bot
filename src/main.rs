mod command;
mod handler;
mod producer;
#[cfg(test)]
mod tests;
mod tokenizer;

use clap::Parser;
use anyhow::Result;
use handler::Handler;
use serenity::{Client, all::GatewayIntents};
use tokio::{fs::File, io::stdin};

#[derive(Parser, Debug)]
#[command(version)]
pub struct Config {
    /// Discord bot token, required
    #[arg(short, long, env="PIPEBOT_DISCORD_TOKEN")]
    pub token: String,

    /// Path to input file, defaults to stdin
    #[arg(short, long, env="PIPEBOT_INPUT_FILE")]
    pub file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::try_parse()?;
    let builder = Client::builder(config.token, GatewayIntents::empty());
    let builder = if let Some(path) = config.file {
        let file = File::open(path).await?;
        builder.event_handler(Handler::new(file))
    } else {
        builder.event_handler(Handler::new(stdin()))
    };
    builder.await?.start().await?;
    Ok(())
}
