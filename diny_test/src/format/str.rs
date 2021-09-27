use core::task::Context;
use diny::backend::{self, Encode as _};
use diny::io;
use crate::Formatter as ThisFormat;

type Data = str;
type ByteEncoder = <ThisFormat as backend::FormatEncode>::EncodeByteSlice;

pub struct Encoder(ByteEncoder);

impl backend::Encode for Encoder
{
    type Data = Data;
    type Format = ThisFormat;

    fn init(data: &Self::Data) -> Self {
        Self(ByteEncoder::init(data.as_bytes()))
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <Self::Format as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        ByteEncoder::start_encode(format, writer, data.as_bytes(), cx)
        .map_pending(Encoder)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<Self::Format as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.0.poll_encode(format, writer, data.as_bytes(), cx)
    }
}

pub type SerializeAll<'w, W> = diny::backend::future::serialize_all::SerializeAll<'w, ThisFormat, W, Data, Encoder>;

pub(crate) fn serialize<'w, W>(format: &'w ThisFormat, writer: &'w mut W, data: &'w Data) -> SerializeAll<'w, W>
where
    W: ::diny::io::AsyncWrite + Unpin,
{
    SerializeAll::new(format, writer, data, Encoder::init(data))
}