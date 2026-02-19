use crate::command::Command;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::net::unix::pipe::{OpenOptions, Receiver};

pub trait LineReader {
    fn read_line<'a>(&'a mut self, buf: &'a mut String) -> impl Future<Output = Result<()>> + Send;
}

pub struct StdinReader<R> {
    inner: BufReader<R>,
}
impl<R: AsyncRead + Send + Unpin> StdinReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: BufReader::new(inner),
        }
    }
}
impl<R: AsyncRead + Send + Unpin> LineReader for StdinReader<R> {
    async fn read_line<'a>(&'a mut self, buf: &'a mut String) -> Result<()> {
        while self.inner.read_line(buf).await? == 0 {}
        Ok(())
    }
}

pub struct FifoReader {
    path: String,
    inner: BufReader<Receiver>,
}
impl FifoReader {
    pub fn new(path: String) -> Result<Self> {
        let inner = BufReader::new(OpenOptions::new().open_receiver(&path)?);
        Ok(Self { path, inner })
    }
}
impl LineReader for FifoReader {
    async fn read_line<'a>(&'a mut self, buf: &'a mut String) -> Result<()> {
        while self.inner.read_line(buf).await? == 0 {
            self.inner = BufReader::new(OpenOptions::new().open_receiver(&self.path)?);
        }
        Ok(())
    }
}

pub struct CommandReader<R> {
    buffer: String,
    inner: R,
}

impl<R: LineReader> CommandReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            buffer: String::new(),
            inner,
        }
    }

    pub async fn next(&mut self) -> Result<Command> {
        self.buffer.clear();
        self.inner.read_line(&mut self.buffer).await?;
        Ok(Command::try_from(
            self.buffer.as_str().trim_end_matches('\n'),
        )?)
    }
}
