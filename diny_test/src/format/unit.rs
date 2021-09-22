use core::task::Context;
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

    fn start_encode_buffer<W>(_format: &Self::Format, _writer: &mut W, _data: &Self::Data, _cx: &mut Context<'_>) -> diny::backend::StartEncodeStatus<Self, <<Self as diny::buffer::BufferEncode>::Format as diny::backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        diny::backend::StartEncodeStatus::Fini
    }

    fn poll_encode_buffer<W>(&mut self, _f: &ThisFormat, _w: &mut W, _cx: &mut Context<'_>) -> diny::backend::PollEncodeStatus<Error>
    where
        W: AsyncWrite + Unpin,
    {
        diny::backend::PollEncodeStatus::Fini
    }
}

pub struct Decoder;

impl diny::backend::Decode for Decoder {
    type Data = Data;
    type Format = ThisFormat;

    fn init() -> Self {
        Decoder
    }

    fn start_decode<R>(_f: &ThisFormat, _r: &mut R, _cx: &mut Context<'_>) -> diny::backend::StartDecodeStatus<Self::Data, Self, Error>
    where
        R: AsyncRead + Unpin,
    {
        diny::backend::StartDecodeStatus::Fini(())
    }

    fn poll_decode<R>(&mut self, _f: &ThisFormat, _r: &mut R, _cx: &mut Context<'_>) -> diny::backend::PollDecodeStatus<Self::Data, Error>
    where
        R: AsyncRead + Unpin,
    {
        diny::backend::PollDecodeStatus::Fini(())
    }
}

serialize_all_def!    (ThisFormat, Data, Encoder);
deserialize_exact_def!(ThisFormat, Data, Decoder);