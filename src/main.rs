mod command;
mod command_reader;
mod discord_context;
mod handler;
#[cfg(test)]
mod tests;
mod tokenizer;

use clap::Parser;
use discord_context::DryRunContext;
use handler::Handler;
use log::LevelFilter;
use serenity::{Client, all::GatewayIntents};
use tokio::io::stdin;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Config {
    /// Set Discord bot token. Required unless --dry-run is enabled
    #[arg(
        short,
        long,
        required = true,
        conflicts_with = "dry_run",
        env = "PIPEBOT_DISCORD_TOKEN"
    )]
    pub token: Option<String>,

    /// Print commands instead of executing them
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

    // TODO: Set level from config
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let handler = Handler::new(stdin());
    if let Some(token) = config.token {
        let builder = Client::builder(token, GatewayIntents::empty()).event_handler(handler);
        builder
            .await
            .expect("Unable to build Discord client")
            .start()
            .await
            .expect("Unable to start Discord client")
    } else {
        handler
            .handle(&DryRunContext)
            .await
            .expect("Unable to handle events")
    }
}
