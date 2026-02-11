use crate::command::Command;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

pub struct CommandReader<R> {
    buffer: String,
    inner: BufReader<R>,
}

impl<R: AsyncRead + Unpin> CommandReader<R> {
    pub fn new(readable: R) -> Self {
        Self {
            buffer: String::new(),
            inner: BufReader::new(readable),
        }
    }

    pub async fn next(&mut self) -> Result<Command> {
        self.buffer.clear();
        while self.inner.read_line(&mut self.buffer).await? == 0 {}
        Ok(Command::try_from(
            self.buffer.as_str().trim_end_matches('\n'),
        )?)
    }
}
