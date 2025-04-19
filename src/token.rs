use anyhow::Result;
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    token: Option<String>,
}

pub fn get_token() -> Result<String> {
    let args = Args::try_parse()?;
    match args.token {
        Some(token) => Ok(token),
        None => Ok(env::var("PIPEBOT_DISCORD_TOKEN")?),
    }
}
