use core::task::Context;
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};
use crate::backend::{self, Encode as _, Decode as _};
use backend::internal::VariantIdx;


type Data<O, E> = Result<O, E>;

pub enum Encode<F, O, E>
where
    F: backend::FormatEncode,
    O: backend::Encodable,
    E: backend::Encodable,
{
    Init,
    Index(VariantIdx, <VariantIdx as backend::Encodable>::Encoder<F>),
    V0(<O as backend::Encodable>::Encoder<F>),
    V1(<E as backend::Encodable>::Encoder<F>),
    Fini,
}

impl<F, O, E> Encode<F, O, E>
where
    F: backend::FormatEncode,
    O: backend::Encodable,
    E: backend::Encodable,
{
    fn variant_index(data: &Data<O, E>) -> VariantIdx {
        match data {
            Ok (_) => 0,
            Err(_) => 1,
        }.into()
    }

    fn after_init<W>(format: &F, writer: &mut W, data: &Data<O, E>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        let index = Self::variant_index(data);
        match <VariantIdx as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &index, cx) {
            backend::StartEncodeStatus::Fini         => Self::after_index(format, writer, data, cx),
            backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self::Index(index, enc)),
            backend::StartEncodeStatus::Error(e)     => backend::StartEncodeStatus::Error(e)
        }
    }

    fn after_index<W>(format: &F, writer: &mut W, data: &Data<O, E>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        match data {
            Data::Ok (d) => Self::ok (format, writer, d, cx),
            Data::Err(e) => Self::err(format, writer, e, cx),
        }
    }

    fn ok<W>(format: &F, writer: &mut W, data: &O, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        <O as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map_pending(Self::V0)
    }

    fn err<W>(format: &F, writer: &mut W, data: &E, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        <E as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map_pending(Self::V1)
    }
}

impl<F, O, E> backend::Encode for Encode<F, O, E>
where
    F: backend::FormatEncode,
    O: backend::Encodable,
    E: backend::Encodable,
{
    type Data = Data<O, E>;
    type Format = F;

    fn init(_data: &Self::Data) -> Self {
        Self::Init
    }

    fn start_encode<W>(format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        Self::after_init(format, writer, data, cx)
    }

    fn poll_encode<W>(&mut self, format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {

        match self {
            Self::Init => encode_chain!(*self, Self::after_init(format, writer, data, cx)),
            Self::Index(idx, enc) => encode_poll_chain!(*self, enc.poll_encode(format, writer, idx, cx), Self::after_index(format, writer, data, cx)),
            Self::V0(enc) =>
                match data {
                    Ok(d) => encode_poll_fini!(*self, enc.poll_encode(format, writer, d, cx)),
                    _ => {
                        *self = Self::Fini;
                        backend::PollEncodeStatus::Error(F::invalid_input_err())
                    }
                },
            Self::V1(enc) =>
                match data {
                    Err(e) => encode_poll_fini!(*self, enc.poll_encode(format, writer, e, cx)),
                    _ => {
                        *self = Self::Fini;
                        backend::PollEncodeStatus::Error(F::invalid_input_err())
                    }
                },
            Self::Fini => backend::PollEncodeStatus::Error(F::invalid_input_err())
        }
    }
}

impl<O, E> backend::Encodable for Data<O, E>
where
    O: backend::Encodable,
    E: backend::Encodable,
{
    type Encoder<F: backend::FormatEncode> = Encode<F, O, E>;
}


impl<O, E> backend::AsyncSerialize for Data<O, E>
where
    O: backend::AsyncSerialize,
    E: backend::AsyncSerialize,
{
    type Future<'w, F, W>
    where
        Self: 'w,
        F: 'w + backend::FormatSerialize,
        W: 'w + AsyncWrite + Unpin,
    = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        F: backend::FormatSerialize,
        W: AsyncWrite + Unpin,

    {
        backend::SerializeAll::new(format, writer, self, <Self::Encoder::<F> as backend::Encode>::init(self))
    }
}


pub enum Decode<F, O, E>
where
    F: backend::FormatDecode,
    O: backend::Decodable,
    E: backend::Decodable,
{
    Init,
    Index(<VariantIdx as backend::Decodable>::Decoder<F>),
    Ok(<O as backend::Decodable>::Decoder<F>),
    Err(<E as backend::Decodable>::Decoder<F>),
    Fini,
}

impl<F, O, E> Decode<F, O, E>
where
    F: backend::FormatDecode,
    O: backend::Decodable,
    E: backend::Decodable,
{
    fn after_init<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<O, E>, Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <VariantIdx as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |idx| Self::after_index(idx, format, reader, cx),
            Self::Index,
        )
    }

    fn after_index<R>(index: VariantIdx, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<O, E>, Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        match *index {
            0 => Self::ok (format, reader, cx),
            1 => Self::err(format, reader, cx),
            _ => backend::StartDecodeStatus::Error(F::invalid_input_err()),
        }
    }

    fn ok<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<O, E>, Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <O as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |o| backend::StartDecodeStatus::Fini(Data::Ok(o)),
            Self::Ok,
        )
    }

    fn err<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<O, E>, Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <E as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |e| backend::StartDecodeStatus::Fini(Data::Err(e)),
            Self::Err,
        )
    }
}

impl<F, O, E> backend::Decode for Decode<F, O, E>
where
    F: backend::FormatDecode,
    O: backend::Decodable,
    E: backend::Decodable,
{
    type Data = Data<O, E>;
    type Format = F;

    fn init() -> Self {
        Self::Init
    }

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        Self::after_init(format, reader, cx)
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        match self {
            Self::Init       => decode_chain!(*self, Self, Self::after_init(format, reader, cx)),
            Self::Index(dec) => decode_poll_chain!(*self, Self, dec.poll_decode(format, reader, cx), |idx| Self::after_index(idx, format, reader, cx)),
            Self::Ok(dec)    => decode_poll_fini!(*self, Self, dec.poll_decode(format, reader, cx), |o| Ok(o)),
            Self::Err(dec)   => decode_poll_fini!(*self, Self, dec.poll_decode(format, reader, cx), Err),
            Self::Fini       => backend::PollDecodeStatus::Error(F::invalid_input_err()),
        }
    }
}

impl<O, E> backend::Decodable for Data<O, E>
where
    O: backend::Decodable,
    E: backend::Decodable,
{
    type Decoder<F: backend::FormatDecode> = Decode<F, O, E>;
}

impl<O, E> backend::AsyncDeserialize for Data<O, E>
where
    O: backend::AsyncDeserialize,
    E: backend::AsyncDeserialize,
{
    type Future<'r, F, R>
    where
        F: 'r + backend::FormatDeserialize,
        R: 'r + AsyncRead + AsyncBufRead + Unpin,
    = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: backend::FormatDeserialize,
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
    }
}