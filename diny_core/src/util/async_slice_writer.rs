use core::{cmp::min, pin::Pin, task::{Context, Poll}};
use crate::io;

#[allow(unused)] // For Doctest
use super::AsyncSliceReader;

/// Writes asynchronously to a slice of bytes, without
/// attempting to allocate more memory if a write is attempted 
/// past the end of the slice.
///
/// Should a write be attempted past the end of the slice without
/// any remaining space, an error of [WriteZero](futures::io::ErrorKind::WriteZero)
/// will be returned.
///
/// # Examples
///
/// ```
/// use futures::{AsyncWriteExt, executor};
/// use diny_core::util::AsyncSliceWriter;
/// 
/// let     send = [7u8; 5];
/// let mut buff = [0u8; 7];
/// let mut writer = AsyncSliceWriter::from(&mut buff[..]);
/// executor::block_on(writer.write(&send));
/// assert_eq!(send, writer.as_written());
/// ```
///
/// The written bytes can then be easily retrieved by using
/// an [AsyncSliceReader]
///
/// ```
/// # use futures::{AsyncWriteExt, executor};
/// # use diny_core::util::AsyncSliceWriter;
/// use futures::AsyncReadExt;
/// use diny_core::util::AsyncSliceReader;
///
/// # let     send = [7u8; 5];
/// # let mut buff = [0u8; 7];
/// # let mut writer = AsyncSliceWriter::from(&mut buff[..]);
/// # executor::block_on(writer.write(&send));
/// let mut recv = [0u8; 5];
/// let mut reader = AsyncSliceReader::from(&writer);
/// executor::block_on(reader.read(&mut recv));
/// assert_eq!(send, recv);
/// ```
pub struct AsyncSliceWriter<'b>{
    buf: &'b mut [u8],
    cur: usize,
}

impl<'b> AsyncSliceWriter<'b> {
    /// Instantiates a new async reader that implements
    /// [AsyncWrite](io::AsyncWrite) over the provided slice.
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, cur: 0 }
    }

    /// Returns the number of bytes that have been written
    /// so far into the provided slice.
    pub fn bytes_written(&self) -> usize {
        self.cur
    }

    /// Returns a read-only slice of the data that has
    /// been written so far into the provided slice.
    pub fn as_written(&self) -> &[u8] {
        &self.buf[..self.cur]
    }

    /// Writes the bytes in `buf` into the provided slice.
    ///
    /// Returns the number of bytes written, which may be
    /// less than `buf.len()` if there is not enough space
    /// remaining in the provided slice.
    fn write_bytes(&mut self, buf: &[u8]) -> usize {
        let n = min(self.buf.len() - self.cur, buf.len());
        if n > 0 {
            self.buf[self.cur..self.cur + n].copy_from_slice(&buf[..n]);
            self.cur += n;
        }
        n
    }
}

impl io::AsyncWrite for AsyncSliceWriter<'_> {
    fn poll_write(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        let n = self.write_bytes(buf);
        if n > 0 {
            Poll::Ready(Ok(n))
        } else {
            Poll::Ready(Err(io::error::write_zero()))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl<'b> From<&'b mut [u8]> for AsyncSliceWriter<'b> {
    fn from(buf: &'b mut [u8]) -> Self {
        Self{ buf, cur: 0 }
    }
}

#[cfg(test)]
mod test {
    use futures::{AsyncWriteExt, executor::block_on};

    use super::*;

    #[test]
    fn can_write_to_slice() {
        const LEN: usize = 10;
        let mut recv = [0u8; LEN];
        let mut writer = AsyncSliceWriter::new(&mut recv);
        assert!(writer.bytes_written() == 0);

        let send = [7u8; LEN];
        let result = block_on(writer.write(&send));        
        assert!(result.is_ok());
        assert!(writer.bytes_written() == LEN);
        assert!(recv == send)
    }
}
