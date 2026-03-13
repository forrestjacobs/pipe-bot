use crate::discord_context::DiscordContext;
use crate::tokenizer::{Tokenizer, TokenizerError, empty_str, nonempty_str};
use serenity::all::{ActivityData, ActivityType, ChannelId};
use std::fmt;
use std::num::ParseIntError;

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
    pub async fn run<C: DiscordContext>(self, ctx: &C) -> serenity::Result<()> {
        match self {
            Command::Message {
                channel_id,
                content,
            } => {
                ctx.say(channel_id, &content).await?;
            }
            Command::Status { name, kind } => {
                ctx.set_activity(Some(ActivityData {
                    name,
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

#[derive(Debug, PartialEq)]
pub enum ParseCommandError {
    ExpectedCommandName,
    ExpectedChannelId(ParseIntError),
    ExpectedMessage,
    ExpectedText,
    ExpectedEnd,
}

impl fmt::Display for ParseCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ParseCommandError::ExpectedCommandName => "expected 'message', 'playing', 'listening_to', 'watching', 'competing_in', or 'clear_status'",
            ParseCommandError::ExpectedChannelId(_) => "expected channel ID",
            ParseCommandError::ExpectedMessage => "expected message",
            ParseCommandError::ExpectedText => "expected text",
            ParseCommandError::ExpectedEnd => "unexpected text",
        })
    }
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = TokenizerError<ParseCommandError>;

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
                _ => return Err(ParseCommandError::ExpectedCommandName),
            })
        })?;

        Ok(match name {
            CommandName::Message => Command::Message {
                channel_id: tokenizer
                    .next(|v| v.parse().map_err(ParseCommandError::ExpectedChannelId))?,
                content: tokenizer
                    .rest(nonempty_str)
                    .map_err(|e| e.map(|_| ParseCommandError::ExpectedMessage))?
                    .to_string(),
            },
            CommandName::Status(kind) => Command::Status {
                kind,
                name: tokenizer
                    .rest(nonempty_str)
                    .map_err(|e| e.map(|_| ParseCommandError::ExpectedText))?
                    .to_string(),
            },
            CommandName::ClearStatus => {
                tokenizer
                    .rest(empty_str)
                    .map_err(|e| e.map(|_| ParseCommandError::ExpectedEnd))?;
                Command::ClearStatus
            }
        })
    }
}
