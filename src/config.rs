use anyhow::Result;
use clap::Parser;

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

pub fn get_config() -> Result<Config> {
    Ok(Config::try_parse()?)
}
