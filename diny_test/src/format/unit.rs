use core::task::Context;
use diny::{backend, buffer};
use crate::Formatter as ThisFormat;

type Error = <ThisFormat as backend::Format>::Error;
type Data = ();

pub struct Encoder;

impl buffer::BufferEncode for Encoder {
    type Data = Data;
    type Format = ThisFormat;

    fn new(_data: &Self::Data) -> Self {
        Encoder
    }

    fn start_encode_buffer<W>(_format: &Self::Format, _writer: &mut W, _data: &Self::Data, _cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, Error> {
        backend::StartEncodeStatus::Fini
    }

    fn poll_encode_buffer<W>(&mut self, _format: &Self::Format, _writer: &mut W, _cx: &mut Context<'_>) -> backend::PollEncodeStatus<Error> {
        backend::PollEncodeStatus::Fini
    }
}

pub struct Decoder;

impl backend::Decode for Decoder {
    type Data = Data;
    type Format = ThisFormat;

    fn init() -> Self {
        Decoder
    }

    fn start_decode<R>(_format: &Self::Format, _reader: &mut R, _cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, Error> {
        backend::StartDecodeStatus::Fini(())
    }

    fn poll_decode<R>(&mut self, _format: &Self::Format, _reader: &mut R, _cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, Error> {
        backend::PollDecodeStatus::Fini(())
    }
}

serialize_all_def!    (ThisFormat, Data, Encoder);
deserialize_exact_def!(ThisFormat, Data, Decoder);