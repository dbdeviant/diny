use core::task::{Context};
use crate::backend::{self, Encode, Format, FormatEncode};
use crate::io;

/// A convenience trait for types that encode to an intermediate
/// buffer prior to serialization.
pub trait BufferEncode: Sized {
    type Format: FormatEncode;
    type Data;

    fn init_buffer(data: &Self::Data) -> Self;

    fn start_encode_buffer<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <<Self as BufferEncode>::Format as Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    ;

    fn poll_encode_buffer<W>(&mut self, format: &Self::Format, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<<Self as BufferEncode>::Format as Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    ;
}

impl<T> Encode for T where T: BufferEncode {
    type Format = T::Format;
    type Data = T::Data;

    fn init(data: &Self::Data) -> Self {
        Self::init_buffer(data)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <<Self as BufferEncode>::Format as Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        <Self as BufferEncode>::start_encode_buffer(format, writer, data, cx)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<<Self as BufferEncode>::Format as Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
     {
        self.poll_encode_buffer(format, writer, cx)
    }
}