use crate::discord_context::DiscordContext;
use crate::tokenizer::{Tokenizer, TokenizerError, empty_str, nonempty_str};
use anyhow::{Context as AnyhowContext, bail};
use serenity::all::{ActivityData, ActivityType, ChannelId};

#[derive(Debug, PartialEq)]
pub enum Command {
    Message {
        channel_id: ChannelId,
        content: String,
    },
    Status {
        name: String,
        kind: ActivityType,
    },
    ClearStatus,
}

impl Command {
    pub async fn run<C: DiscordContext>(self, ctx: &C) -> anyhow::Result<()> {
        match self {
            Command::Message {
                channel_id,
                content,
            } => {
                ctx.say(channel_id, &content).await?;
            }
            Command::Status { name, kind } => {
                ctx.set_activity(Some(ActivityData {
                    name: name.into(),
                    kind,
                    state: None,
                    url: None,
                }));
            }
            Command::ClearStatus => {
                ctx.set_activity(None);
            }
        }
        Ok(())
    }
}

enum CommandName {
    Message,
    Status(ActivityType),
    ClearStatus,
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = TokenizerError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut tokenizer = Tokenizer::from(value);

        let name = tokenizer.next(|v| {
            Ok(match v {
                "message" => CommandName::Message,
                "playing" => CommandName::Status(ActivityType::Playing),
                "listening_to" => CommandName::Status(ActivityType::Listening),
                "watching" => CommandName::Status(ActivityType::Watching),
                "competing_in" => CommandName::Status(ActivityType::Competing),
                "clear_status" => CommandName::ClearStatus,
                _ => bail!("expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'"),
            })
        })?;

        Ok(match name {
            CommandName::Message => Command::Message {
                channel_id: tokenizer.next(|v| v.parse().context("expected channel ID"))?,
                content: tokenizer.rest(nonempty_str("message"))?.to_string(),
            },
            CommandName::Status(kind) => Command::Status {
                kind,
                name: tokenizer.rest(nonempty_str("text"))?.to_string(),
            },
            CommandName::ClearStatus => {
                tokenizer.rest(empty_str)?;
                Command::ClearStatus
            }
        })
    }
}
