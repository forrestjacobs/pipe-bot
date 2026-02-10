use crate::command::Command;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

pub struct Producer<R> {
    buffer: String,
    reader: BufReader<R>,
}

impl<R: AsyncRead + Unpin> Producer<R> {
    pub fn new(readable: R) -> Self {
        Self {
            buffer: String::new(),
            reader: BufReader::new(readable),
        }
    }

    pub async fn next(&mut self) -> Result<Command> {
        self.buffer.clear();
        while self.reader.read_line(&mut self.buffer).await? == 0 {}
        Ok(Command::try_from(
            self.buffer.as_str().trim_end_matches('\n'),
        )?)
    }
}
