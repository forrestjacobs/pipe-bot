mod command;
mod command_reader;
mod discord_context;
mod handler;
#[cfg(test)]
mod tests;
mod tokenizer;

use clap::Parser;
use command_reader::{FifoReader, LineReader, StdinReader};
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

    /// Set input file. Defaults to stdin
    #[arg(short, long, env = "PIPEBOT_INPUT_FILE")]
    pub file: Option<String>,

    /// Print commands instead of executing them
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

async fn start_with_reader<R: LineReader + Send + 'static>(reader: R, token: Option<String>) {
    let handler = Handler::new(reader);
    if let Some(token) = token {
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

#[tokio::main]
async fn main() {
    let config = Config::parse();

    // TODO: Set level from config
    env_logger::builder().filter_level(LevelFilter::Info).init();

    if let Some(path) = config.file {
        start_with_reader(
            FifoReader::new(path).expect("Unable to read file"),
            config.token,
        )
        .await
    } else {
        start_with_reader(StdinReader::new(stdin()), config.token).await
    }
}
