#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::task::Context;
use diny::{backend, io};
use crate::Formatter as ThisFormat;

type Data = String;
type StrEncoder = <ThisFormat as backend::FormatEncode>::EncodeStr;
type ByteVecDecoder = <ThisFormat as backend::FormatDecode>::DecodeByteVec;
type Error = <ThisFormat as backend::Format>::Error;

pub struct Encoder(StrEncoder);

impl backend::Encode for Encoder
{
    type Data = Data;
    type Format = ThisFormat;

    fn init(data: &Self::Data) -> Self {
        Self(StrEncoder::init(data))
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        StrEncoder::start_encode(format, writer, data, cx)
        .map_pending(Encoder)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.0.poll_encode(format, writer, data, cx)
    }
}

pub type SerializeAll<'w, W> = diny::backend::future::serialize_all::SerializeAll<'w, ThisFormat, W, Data, Encoder>;

#[allow(clippy::ptr_arg)]
pub(crate) fn serialize<'w, W>(format: &'w ThisFormat, writer: &'w mut W, data: &'w Data) -> SerializeAll<'w, W>
where
    W: ::diny::io::AsyncWrite + Unpin,
{
    SerializeAll::new(format, writer, data, <Encoder as backend::Encode>::init(data))
}


pub struct Decoder(ByteVecDecoder);

fn into_string(vec: Vec<u8>) -> Result<String, Error> {
    String::from_utf8(vec).map_err(|_| <ThisFormat as backend::Format>::invalid_data_err())
}

impl backend::Decode for Decoder {
    type Format = ThisFormat;
    type Data = Data;

    fn init() -> Self {
        Self(ByteVecDecoder::init())
    }

    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <<Self as backend::Decode>::Format as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        ByteVecDecoder::start_decode(format, reader, cx)
        .and_then(
            |d| into_string(d).into(),
            Self,
        )
    }

    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <<Self as backend::Decode>::Format as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
     {
         self.0.poll_decode(format, reader, cx)
         .and_then(|d| into_string(d).into())
    }
}

pub type DeserializeExact<'r, R> = backend::future::deserialize_exact::DeserializeExact<'r, ThisFormat, R, Data, Decoder>;

pub(crate) fn deserialize<'r, R>(format: &'r ThisFormat, reader: &'r mut R) -> DeserializeExact<'r, R>
where
    R: diny::io::AsyncRead + diny::io::AsyncBufRead + Unpin,
{
    backend::DeserializeExact::new(format, reader, <Decoder as backend::Decode>::init())
}