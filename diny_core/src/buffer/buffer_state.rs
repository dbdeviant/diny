use core::task::Context;

use crate::buffer::BufferCursor;
use crate::{backend, io};


/// A convenient struct to represent serialization of an underlying
/// byte buffer.
///
/// Generally useful for primitive times (e.g. u64, str) that need
/// to serialize multiple bytes using a predetermined buffer.
pub struct BufferState<const LEN: usize> {
    pub cur: BufferCursor,
    buf: [u8; LEN],
}

impl<const LEN: usize> BufferState<LEN> {
    /// Initialize a new buffer state with the contents of
    /// the provided `buf`.
    #[inline(always)]
    pub fn with_contents(buf: [u8; LEN]) -> Self {
        Self {
            cur: BufferCursor::new(&buf),
            buf,
        }
    }

    /// Initialize a new buffer state with the contents of
    /// the first `n` bytes of the provided `buf`.
    #[inline(always)]
    pub fn with_partial_contents(buf: [u8; LEN], n: usize) -> Self {
        Self {
            cur: BufferCursor::new(&buf[..n]),
            buf,
        }
    }

    /// Initialize a new buffer state and zeroize its contents.
    #[inline(always)]
    pub fn init() -> Self {
        Self::with_contents([0u8; LEN])
    }

    /// Return a slice referencing the entire contents of the buffer.
    #[inline(always)]
    pub fn buffer(&self) -> &[u8; LEN] {
        &self.buf
    }

    /// Return whether or not more data is expected to be processed.
    #[inline(always)]
    pub fn is_pending(&self) -> bool {
        self.cur.is_pending()
    }

    /// Return the number of bytes that have been processed so far.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.cur.len()
    }

    /// Return whether or not any bytes have been processed yet.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.cur.is_empty()
    }

    /// Extend the size of the interal buffer window to acoomodate more data.
    ///
    /// This will only extend the buffer up to the size of the pre-allocated
    /// buffer.  It is intended to be used when the total number of bytes is
    /// not known ahead of time and will be deserialized as part of the buffer
    /// transfer.
    #[inline(always)]
    pub fn extend_len(&mut self, n: usize) {
        debug_assert!(LEN - self.cur.len() >= n);
        self.cur.extend_len(::core::cmp::min(LEN - self.cur.len(), n));
    }

    pub fn start_write<W>(&mut self, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<io::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.cur.start_write(writer, &self.buf, cx)
    }

    /// Attempt to write all remaining bytes from the buffer.
    pub fn write_remaining<W>(&mut self, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<io::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.cur.write_remaining(writer, &self.buf, cx)
    }

    pub fn start_read<R>(&mut self, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<(), io::Error>
    where
        R: io::AsyncRead + Unpin,
    {
        self.cur.start_read(reader, &mut self.buf, cx)
    }

    /// Attempt to read all remaining bytes into the buffer.
    pub fn read_remaining<R>(&mut self, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<(), io::Error>
    where
        R: io::AsyncRead + Unpin,
    {
        self.cur.read_remaining(reader, &mut self.buf, cx)
    }
}