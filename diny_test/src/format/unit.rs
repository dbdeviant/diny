use core::{task::{Context, Poll}};
use futures::{AsyncRead, AsyncWrite};

use crate::Formatter as ThisFormat;

type Error = <ThisFormat as diny::backend::Format>::Error;

type Data = ();

pub struct Encoder;

impl diny::buffer::BufferEncode for Encoder {
    type Data = Data;
    type Format = ThisFormat;

    fn new(_data: &Self::Data) -> Self {
        Encoder
    }

    fn poll_encode_buffer<W>(&mut self, _f: &ThisFormat, _w: &mut W, _cx: &mut Context<'_>) -> Poll<Result<(), Error>>
    where
        W: AsyncWrite + Unpin,
    {
        Poll::Ready(Ok(()))
    }
}

pub struct Decoder;

impl diny::backend::Decode for Decoder {
    type Data = Data;
    type Format = ThisFormat;

    fn init() -> Self {
        Decoder
    }

    fn start_decode<R>(_f: &ThisFormat, _r: &mut R, _cx: &mut Context<'_>) -> Result<diny::backend::DecodeStatus<Self::Data, Self>, Error>
    where
        R: AsyncRead + Unpin,
    {
        Ok(diny::backend::DecodeStatus::Ready(()))
    }

    fn poll_decode<R>(&mut self, _f: &ThisFormat, _r: &mut R, _cx: &mut Context<'_>) -> Poll<Result<Self::Data, Error>>
    where
        R: AsyncRead + Unpin,
    {
        Poll::Ready(Ok(()))
    }
}

serialize_all_def!    (ThisFormat, Data, Encoder);
deserialize_exact_def!(ThisFormat, Data, Decoder);