use anyhow::{Context, Result};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    token: Option<String>,
}

fn get_token_with_args(args: Args) -> Result<String> {
    match args.token {
        Some(token) => Ok(token),
        None => Ok(env::var("PIPEBOT_DISCORD_TOKEN").context("token not found")?),
    }
}

pub fn get_token() -> Result<String> {
    get_token_with_args(Args::try_parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn clear_env() {
        unsafe {
            env::remove_var("PIPEBOT_DISCORD_TOKEN");
        }
    }

    #[test]
    #[serial]
    fn get_token_from_args() {
        clear_env();
        unsafe {
            env::set_var("PIPEBOT_DISCORD_TOKEN", "ENV_TOKEN");
        }
        assert_eq!(
            get_token_with_args(Args {
                token: Some("ARG_TOKEN".to_string())
            })
            .unwrap(),
            "ARG_TOKEN".to_string()
        );
    }

    #[test]
    #[serial]
    fn get_token_from_env() {
        clear_env();
        unsafe {
            env::set_var("PIPEBOT_DISCORD_TOKEN", "ENV_TOKEN");
        }
        assert_eq!(
            get_token_with_args(Args { token: None }).unwrap(),
            "ENV_TOKEN".to_string()
        );
    }

    #[test]
    #[serial]
    fn missing_token() {
        clear_env();
        assert_eq!(
            get_token_with_args(Args { token: None })
                .unwrap_err()
                .to_string(),
            "token not found"
        );
    }
}
