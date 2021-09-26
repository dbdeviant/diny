use core::{cmp::min, pin::Pin, task::{Context, Poll}};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

#[allow(unused)]
use crate::{backend, io};
use io::{AsyncWrite as _, AsyncRead as _, AsyncBufRead as _};


/// Retains the current index state into a serialization buffer.
///
/// Typically used by a [BufferState](super::BufferState)
/// to keep track of which bytes still require processing.
pub struct BufferCursor {
    offset: usize,
    len: usize,
}

impl BufferCursor {
    /// Instantiate a new cursor over the beginning of the provided `buf`.
    #[inline(always)]
    pub fn new(buf: &[u8]) -> Self {
        Self::with_len(buf.len())
    }

    /// Instantiate a new cursor representing the beginning of a buffer
    /// with a specified `len`.
    #[inline(always)]
    pub fn with_len(len: usize) -> Self {
        Self {
            offset: 0,
            len,
        }
    }

    /// Return the number of bytes the cursor has processed so far.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return whether or not any bytes have been processed yet.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Return whether or not more data is expected to be processed.
    #[inline(always)]
    pub fn is_pending(&self) -> bool {
        self.offset < self.len
    }

    pub(crate) fn extend_len(&mut self, n: usize) {
        self.len += n;
    }

    fn is_error(&self) -> bool {
        self.len == 0
    }

    fn mark_as_error(&mut self) {
        self.len = 0;
    }

    fn remaining(&self) -> usize {
        self.len - self.offset
    }

    fn advance(&mut self, n: usize) {
        self.offset += min(n, self.remaining());
    }

    pub fn start_write<W>(&mut self, mut writer: &mut W, data: &[u8], cx: &mut Context<'_>) -> backend::PollEncodeStatus<io::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        debug_assert!(!self.is_error() && self.is_pending() && self.len <= data.len());
        while self.is_pending() {
            match Pin::new(&mut writer).poll_write(cx, &data[self.offset..self.len]) {
                Poll::Ready(r) =>  match r {
                    Ok(n) => {
                        if n == 0 {
                            self.mark_as_error();
                            return backend::PollEncodeStatus::Error(io::error::write_zero());
                        } else {
                            self.advance(n);
                        }
                    },
                    Err(e) => {
                        self.mark_as_error();
                        return backend::PollEncodeStatus::Error(e);
                    }
                }
                Poll::Pending => return backend::PollEncodeStatus::Pending,
            }
        }

        backend::PollEncodeStatus::Fini
    }

    /// Attempt to write all of the bytes in `data`, starting from the current
    /// offset of the cursor.
    ///
    /// This implementation differs from a typical [AsyncWrite] operation in that
    /// the contents of data are expected to be unchanging from one call to the
    /// next, despite progress being made.  The expectation is that the caller
    /// does not need to retain any information about the progress of the write,
    /// and simply needs to pass in the same references for each call.
    pub fn write_remaining<W>(&mut self, writer: &mut W, data: &[u8], cx: &mut Context<'_>) -> backend::PollEncodeStatus<io::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        debug_assert!(!self.is_error() && self.is_pending() && self.len <= data.len());
        if self.len > data.len() {
            self.mark_as_error();
        }

        if self.is_error() {
            backend::PollEncodeStatus::Error(io::error::invalid_input())
        } else {
            self.start_write(writer, data, cx)
        }
    }

    pub fn start_read<R>(&mut self, mut reader: &mut R, data: &mut [u8], cx: &mut Context<'_>) -> backend::PollDecodeStatus<(), io::Error>
    where
        R: io::AsyncRead + Unpin,
    {
        debug_assert!(!self.is_error() && self.is_pending() && self.len <= data.len());
        loop {
            match Pin::new(&mut reader).poll_read(cx, &mut data[self.offset..self.len]) {
                Poll::Ready(r) => match r {
                    Ok(n) => {
                        if n == 0 {
                            self.mark_as_error();
                            return backend::PollDecodeStatus::Error(io::error::unexpected_eof());
                        } else {
                            self.advance(n);
                            if !self.is_pending() {
                                return backend::PollDecodeStatus::Fini(());
                            }
                        }
                    },
                    Err(e) => {
                        self.mark_as_error();
                        return backend::PollDecodeStatus::Error(e);
                    }
                },
                Poll::Pending => return backend::PollDecodeStatus::Pending,
            }
        }        
    }
    
    /// Attempt to read all remaining bytes that are expected into `data`, starting
    /// from the current offset of the cursor.
    ///
    /// This implementation differs from a typical [AsyncRead] operation in that
    /// the contents of data are expected to be unchanging from one call to the
    /// next, despite progress being made.  The expectation is that the caller
    /// does not need to retain any information about the progress of the read,
    /// and simply needs to pass in the same references for each call.
    pub fn read_remaining<R>(&mut self, reader: &mut R, data: &mut [u8], cx: &mut Context<'_>) -> backend::PollDecodeStatus<(), io::Error>
    where
        R: io::AsyncRead + Unpin,
    {
        debug_assert!(!self.is_error() && self.is_pending() && self.len <= data.len());
        if self.len > data.len() {
            self.mark_as_error();
        }

        if self.is_error() {
            backend::PollDecodeStatus::Error(io::error::invalid_input())
        } else {
            self.start_read(reader, data, cx)
        }
    }

    /// Semantically equivalent to `read_remaining`, only this method takes advantage
    /// of the [AsyncBufRead] trait to minimize the number of copies required to transfer
    /// the bytes into a pre-allocated [Vec].
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn fill_vec<R>(&mut self, mut reader: &mut R, data: &mut Vec<u8>, cx: &mut Context<'_>) -> backend::PollDecodeStatus<(), io::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        debug_assert!(!self.is_error() && self.is_pending());

        if self.is_error() {
            return backend::PollDecodeStatus::Error(io::error::invalid_input());            
        }

        while self.is_pending() {
            match Pin::new(&mut reader).poll_fill_buf(cx) {
                Poll::Ready(r) => match r {
                    Ok(buf) => {
                        if buf.is_empty() {
                            self.mark_as_error();
                            return backend::PollDecodeStatus::Error(io::error::unexpected_eof());
                        } else {
                            let n = min(buf.len(), self.remaining());
                            data.extend_from_slice(&buf[..n]);
                            Pin::new(&mut reader).consume(n);            
                            self.advance(n);
                        }
                    },
                    Err(err) => {
                        self.mark_as_error();
                        return backend::PollDecodeStatus::Error(err);
                    }
                },
                Poll::Pending => return backend::PollDecodeStatus::Pending,
            }
        }

        backend::PollDecodeStatus::Fini(())
    }
}