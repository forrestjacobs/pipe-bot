use crate::command::{Command, ParseCommandError};
use crate::tokenizer::TokenizerError;
use std::{error, fmt};
use tokio::io::{self, AsyncBufReadExt, AsyncRead, BufReader};

pub struct CommandReader<R> {
    buffer: String,
    inner: R,
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Parse(TokenizerError<ParseCommandError>),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "I/O error: {e}"),
            ReadError::Parse(e) => write!(f, "Unable to parse command:\n{e}"),
        }
    }
}

impl error::Error for ReadError {}

impl<R: AsyncRead + Unpin> CommandReader<BufReader<R>> {
    pub fn new(inner: R) -> Self {
        Self {
            buffer: String::new(),
            inner: BufReader::new(inner),
        }
    }

    pub async fn next(&mut self) -> Result<Command, ReadError> {
        self.buffer.clear();
        while self
            .inner
            .read_line(&mut self.buffer)
            .await
            .map_err(ReadError::Io)?
            == 0
        {}
        Ok(
            Command::try_from(self.buffer.as_str().trim_end_matches('\n'))
                .map_err(ReadError::Parse)?,
        )
    }
}
