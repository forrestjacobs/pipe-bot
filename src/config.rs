use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Config {
    #[arg(short, long, env="PIPEBOT_DISCORD_TOKEN")]
    pub token: String,

    #[arg(short, long, env="PIPEBOT_INPUT_FILE")]
    pub file: Option<String>,
}

pub fn get_config() -> Result<Config> {
    Ok(Config::try_parse()?)
}
