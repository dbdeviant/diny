use core::{cmp::min, pin::Pin, task::{Context, Poll}};
use crate::io;

/// Reads asynchronously from a slice of bytes, without
/// attempting to acquire any more data if a read is attempted
/// past the end of the slice.
///
/// # Examples
///
/// ```
/// use futures::{AsyncReadExt, executor};
/// use diny_core::util::AsyncSliceReader;
/// 
/// const LEN: usize = 10;
/// let send     = [7u8; LEN];
/// let mut recv = [0u8; LEN];
/// let mut reader = AsyncSliceReader::from(&send[..]);
/// executor::block_on(reader.read(&mut recv));
/// assert_eq!(send, recv);
/// ```
pub struct AsyncSliceReader<'b>{
    buf: &'b [u8],
    cur: usize,
}

impl Unpin for AsyncSliceReader<'_> {}

impl<'b> AsyncSliceReader<'b> {
    /// Instantiates a new async reader that implements
    /// [AsyncRead] and [AsyncBufRead] over the provided
    /// slice.
    pub fn new(buf: &'b [u8]) -> Self {
        Self { buf, cur: 0 }
    }

    /// Returns the number of bytes that have been read
    /// so far from the provided slice.
    pub fn bytes_read(&self) -> usize {
        self.cur
    }

    fn consume(&mut self, amt: usize) {
        self.cur += min(self.bytes_available(), amt);
    }

    fn bytes_available(&self) -> usize {
        self.buf.len() - self.cur
    }

    fn read_into(&mut self, buf: &mut [u8]) -> usize {
        let n = min(self.bytes_available(), buf.len());
        if n > 0 {
            buf[..n].copy_from_slice(&self.buf[self.cur..self.cur + n]);
            self.cur += n;
        }
        n
    }
}

impl io::AsyncRead for AsyncSliceReader<'_> {
    fn poll_read(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(self.read_into(buf)))
    }
}

impl io::AsyncBufRead for AsyncSliceReader<'_>{
    fn poll_fill_buf(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        Poll::Ready(Ok(&self.buf[self.cur..]))
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        (&mut *self).consume(amt);
    }
}


impl<'b> From<&'b [u8]> for AsyncSliceReader<'b> {
    fn from(buf: &'b [u8]) -> Self {
        Self{ buf, cur: 0 }
    }
}

impl<'b> From<&'b super::AsyncSliceWriter<'b>> for AsyncSliceReader<'b> {
    fn from(s: &'b super::AsyncSliceWriter<'b>) -> Self {
        Self{ buf: s.as_written(), cur: 0 }
    }
}

#[cfg(test)]
mod test {
    use futures::{AsyncReadExt, executor::block_on};

    use super::*;

    #[test]
    fn can_read_from_vec() {
        const LEN: usize = 10;
        let send = [7u8; LEN];
        let mut reader = AsyncSliceReader::new(&send);
        assert!(reader.bytes_read() == 0);

        let mut recv = [0u8; LEN];
        let result = block_on(reader.read(&mut recv));        
        assert!(result.is_ok());
        assert!(reader.bytes_read() == LEN);
        assert!(recv == send)
    }
}
