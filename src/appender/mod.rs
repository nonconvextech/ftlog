//! Useful appenders
pub mod file;
pub mod udp;

pub use file::{FileAppender, Period};
use std::io::Write;
pub use time::Duration;

/// Chain multiple appenders
///
/// This can help when you want to log the same content to multiple destinations
pub struct ChainAppenders {
    writers: Vec<Box<dyn Write + Send>>,
}

impl ChainAppenders {
    pub fn new(writers: Vec<Box<dyn Write + Send>>) -> Self {
        Self { writers }
    }
}
impl Write for ChainAppenders {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for writer in &mut self.writers {
            writer.write_all(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }
        Ok(())
    }
}
