use core::task::Context;
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};    
use diny::buffer::BufferState;

use crate::Formatter as ThisFormat;

type Error = <ThisFormat as diny::backend::Format>::Error;

type Data = bool;
const BUF_SIZE: usize = 1;

const TRUE:  u8 = 1;
const FALSE: u8 = 0;

fn to_le_bytes(v: Data) -> [u8; 1] {
    match v {
        true  => [TRUE],
        false => [FALSE],
    }
}

fn from_le_bytes(v: [u8; 1]) -> futures::io::Result<Data> {
    match v[0] {
        TRUE  => Ok(true),
        FALSE => Ok(false),
        _ => Err(<ThisFormat as diny::backend::Format>::invalid_data_err()),
    }
}

pub struct Encoder(BufferState<BUF_SIZE>);

impl Encoder {
    fn new(data: &Data) -> Self {
        Encoder(BufferState::with_contents(to_le_bytes(*data)))
    }
}

impl diny::buffer::BufferEncode for Encoder {
    type Data = Data;
    type Format = ThisFormat;

    fn new(data: &Self::Data) -> Self {
        Self::new(data)
    }

    fn start_encode_buffer<W>(_format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> diny::backend::StartEncodeStatus<Self, <<Self as diny::buffer::BufferEncode>::Format as diny::backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        let mut enc = Self::new(data);
        match enc.0.start_write(writer, cx) {
            diny::backend::PollEncodeStatus::Fini => diny::backend::StartEncodeStatus::Fini,
            diny::backend::PollEncodeStatus::Pending => diny::backend::StartEncodeStatus::Pending(enc),
            diny::backend::PollEncodeStatus::Error(err) => diny::backend::StartEncodeStatus::Error(err),
        }
    }

    fn poll_encode_buffer<W>(&mut self, _format: &Self::Format, writer: &mut W, cx: &mut Context<'_>) -> diny::backend::PollEncodeStatus<Error>
    where
        W: AsyncWrite + Unpin,
    {
        self.0.write_remaining(writer, cx)
    }
}

pub struct Decoder(BufferState<BUF_SIZE>);

impl diny::backend::Decode for Decoder {
    type Data = Data;
    type Format = ThisFormat;

    fn init() -> Self {
        Self(BufferState::init())
    }

    fn start_decode<R>(f: &ThisFormat, r: &mut R, cx: &mut Context<'_>) -> diny::backend::StartDecodeStatus<Self::Data, Self, Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        let mut decode = Self::init();
        decode
        .poll_decode(f, r, cx)
        .lift(decode)
    }

    fn poll_decode<R>(&mut self, _format: &ThisFormat, reader: &mut R, cx: &mut Context<'_>) -> diny::backend::PollDecodeStatus<Self::Data, Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        (&mut self.0)
        .read_remaining(reader, cx)
        .and_then(|()| from_le_bytes(*self.0.buffer()).into())
    }
}

serialize_all_def!    (ThisFormat, Data, Encoder);
deserialize_exact_def!(ThisFormat, Data, Decoder);