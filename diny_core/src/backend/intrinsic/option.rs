use core::task::Context;
use crate::backend::{self, Encode as _, Decode as _};
use crate::io;
use backend::internal::VariantIdx;


type Data<T> = Option<T>;

pub enum Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    Init,
    Index(VariantIdx, <VariantIdx as backend::Encodable>::Encoder<F>),
    V0(<() as backend::Encodable>::Encoder<F>),
    V1(<T as backend::Encodable>::Encoder<F>),
    Fini,
}

impl<F, T> Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    fn variant_index(data: &Data<T>) -> VariantIdx {
        match data {
            None    => 0,
            Some(_) => 1,
        }.into()
    }

    fn after_init<W>(format: &F, writer: &mut W, data: &Data<T>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        let index = Self::variant_index(data);
        match <VariantIdx as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &index, cx) {
            backend::StartEncodeStatus::Fini         => Self::after_index(format, writer, data, cx),
            backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self::Index(index, enc)),
            backend::StartEncodeStatus::Error(e)     => backend::StartEncodeStatus::Error(e)
        }
    }

    fn after_index<W>(format: &F, writer: &mut W, data: &Data<T>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        match data {
            Data::None    => Self::none(format, writer, &(), cx),
            Data::Some(d) => Self::some(format, writer, d, cx),
        }
    }

    fn none<W>(format: &F, writer: &mut W, data: &(), cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        <() as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map_pending(Self::V0)
    }

    fn some<W>(format: &F, writer: &mut W, data: &T, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        <T as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map_pending(Self::V1)
    }
}



impl<F, T> backend::Encode for Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Data = Data<T>;
    type Format = F;

    fn init(_data: &Self::Data) -> Self {
        Self::Init
    }

    fn start_encode<W>(format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        Self::after_init(format, writer, data, cx)
    }

    fn poll_encode<W>(&mut self, format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        match self {
            Self::Init            => encode_chain!(*self, Self::after_init(format, writer, data, cx)),
            Self::Index(idx, enc) => encode_poll_chain!(*self, enc.poll_encode(format, writer, idx, cx), Self::after_index(format, writer, data, cx)),
            Self::V0(enc)         => encode_poll_fini!(*self, enc.poll_encode(format, writer, &(), cx)),
            Self::V1(enc)         =>
                match data {
                    Some(d) => encode_poll_fini!(*self, enc.poll_encode(format, writer, d, cx)),
                    None => {
                        *self = Self::Fini;
                        backend::PollEncodeStatus::Error(F::invalid_input_err())
                    }
                },
            Self::Fini => backend::PollEncodeStatus::Error(F::invalid_input_err())
        }
    }
}

impl<T> backend::Encodable for Data<T>
where
    T: backend::Encodable,
{
    type Encoder<F: backend::FormatEncode> = Encode<F, T>;
}


impl<T> backend::AsyncSerialize for Data<T>
where
    T: backend::AsyncSerialize,
{
    type Future<'w, F, W>
    where
        Self: 'w,
        F: 'w + backend::FormatSerialize,
        W: 'w + io::AsyncWrite + Unpin,
    = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        F: backend::FormatSerialize,
        W: io::AsyncWrite + Unpin,

    {
        backend::SerializeAll::new(format, writer, self, <Self::Encoder::<F> as backend::Encode>::init(self))
    }
}


pub enum Decode<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    Init,
    Index(<VariantIdx as backend::Decodable>::Decoder<F>),
    None(<() as backend::Decodable>::Decoder<F>),
    Some(<T as backend::Decodable>::Decoder<F>),
    Fini,
}

impl<F, T> Decode<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    fn after_init<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<T>, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        <VariantIdx as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |idx| Self::after_index(idx, format, reader, cx),
            Self::Index,
        )
    }

    fn after_index<R>(index: VariantIdx, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<T>, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        match *index {
            0 => Self::none(format, reader, cx),
            1 => Self::some(format, reader, cx),
            _ => backend::StartDecodeStatus::Error(F::invalid_input_err()),
        }
    }

    fn none<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<T>, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        <() as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |()| backend::StartDecodeStatus::Fini(None),
            Self::None,
        )
    }

    fn some<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Data<T>, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        <T as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |s| backend::StartDecodeStatus::Fini(Some(s)),
            Self::Some,
        )
    }
}

impl<F, T> backend::Decode for Decode<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    type Data = Data<T>;
    type Format = F;

    fn init() -> Self {
        Self::Init
    }

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        Self::after_init(format, reader, cx)
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        match self {
            Self::Init       => decode_chain!(*self, Self, Self::after_init(format, reader, cx)),
            Self::Index(dec) => decode_poll_chain!(*self, Self, dec.poll_decode(format, reader, cx), |idx| Self::after_index(idx, format, reader, cx)),
            Self::None(dec)  => decode_poll_fini!(*self, Self, dec.poll_decode(format, reader, cx), |_| None),
            Self::Some(dec)  => decode_poll_fini!(*self, Self, dec.poll_decode(format, reader, cx), Some),
            Self::Fini       => backend::PollDecodeStatus::Error(F::invalid_input_err()),
        }
    }
}

impl<T> backend::Decodable for Data<T>
where
    T: backend::Decodable,
{
    type Decoder<F: backend::FormatDecode> = Decode<F, T>;
}

impl<T> backend::AsyncDeserialize for Data<T>
where
    T: backend::AsyncDeserialize,
{
    type Future<'r, F, R>
    where
        F: 'r + backend::FormatDeserialize,
        R: 'r + io::AsyncRead + io::AsyncBufRead + Unpin,
    = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: backend::FormatDeserialize,
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
    }
}