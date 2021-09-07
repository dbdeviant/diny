use core::task::{Context, Poll};
use futures::AsyncWrite;
use crate::backend::{Encode, Format, FormatEncode};

/// A convenience trait for types that encode to an intermediate
/// buffer prior to serialization.
pub trait BufferEncode {
    type Format: FormatEncode;
    type Data;

    fn new(data: &Self::Data) -> Self;

    fn poll_encode_buffer<W>(&mut self, format: &Self::Format, writer: &mut W, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as BufferEncode>::Format as Format>::Error>>
    where
        W: AsyncWrite + Unpin,
    ;
}

impl<T> Encode for T where T: BufferEncode {
    type Format = T::Format;
    type Data = T::Data;

    fn init(data: &Self::Data) -> Self {
        Self::new(data)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <<Self as Encode>::Format as Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        let mut encode = Self::new(data);
        match encode.poll_encode_buffer(format, writer, cx) {
            Poll::Ready(r) => r.map(|_| None),
            Poll::Pending => Ok(Some(encode)),
        }
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as BufferEncode>::Format as Format>::Error>>
    where
        W: AsyncWrite + Unpin,
     {
        self.poll_encode_buffer(format, writer, cx)
    }
}