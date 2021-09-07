use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};
use crate::backend::{self, Encode as _, Decode as _};
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

    fn after_init<W>(format: &F, writer: &mut W, data: &Data<T>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        let index = Self::variant_index(data);
        <VariantIdx as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &index, cx)
        .and_then(|o| match o {
            Some(s) => Ok(Self::Index(index, s)),
            None    => Self::after_index(format, writer, data, cx),
        })
    }

    fn after_index<W>(format: &F, writer: &mut W, data: &Data<T>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        match data {
            Data::None    => Self::none(format, writer, &(), cx),
            Data::Some(d) => Self::some(format, writer, d, cx),
        }
    }

    fn none<W>(format: &F, writer: &mut W, data: &(), cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        <() as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map(|o| match o {
            None    => Self::Fini,
            Some(s) => Self::V0(s),
        })
    }

    fn some<W>(format: &F, writer: &mut W, data: &T, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        <T as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map(|o| match o {
            None    => Self::Fini,
            Some(s) => Self::V1(s),
        })
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

    fn start_encode<W>(format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        Self::after_init(format, writer, data, cx)
        .map(|s| match s {
            Self::Fini => None,
            _          => Some(s),
        })
    }

    fn poll_encode<W>(&mut self, format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as backend::Encode>::Format as backend::Format>::Error>>
    where
        W: AsyncWrite + Unpin,
    {
        let res = match self {
            Self::Init => {
                Self::after_init(format, writer, data, cx)
            },
            Self::Index(idx, enc) => {
                futures::ready!(enc.poll_encode(format, writer, idx, cx))
                .and_then(|_| Self::after_index(format, writer, data, cx))
            }
            Self::V0(enc) => {
                match data {
                    Data::None => futures::ready!(enc.poll_encode(format, writer, &(), cx)).map(|_| Self::Fini),
                    _ => Err(F::invalid_input_err()),
                }
            }
            Self::V1(enc) => {
                match data {
                    Data::Some(d) => futures::ready!(enc.poll_encode(format, writer, d, cx)).map(|_| Self::Fini),
                    _ => Err(F::invalid_input_err()),
                }
            }
            Self::Fini => {
                Err(F::invalid_input_err())
            }
        };

        match res {
            Ok(enc) => {
                *self = enc;
                match self {
                    Self::Fini => Poll::Ready(Ok(())),
                    _          => Poll::Pending,
                }
            },
            Err(e) => {
                *self = Self::Fini;
                Poll::Ready(Err(e))
            }
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
        F: 'w + backend::FormatSerialize<'w>,
        W: 'w + AsyncWrite + Unpin,
    = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        Self: 'w,
        F: backend::FormatSerialize<'w>,
        W: AsyncWrite + Unpin,

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
    fn after_init<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<T>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <VariantIdx as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(|status| match status {
            backend::DecodeStatus::Ready(idx) => Self::after_index(&idx, format, reader, cx),
            backend::DecodeStatus::Pending(p) => Ok(backend::DecodeStatus::Pending(Self::Index(p))),
        })
    }

    fn after_index<R>(index: &VariantIdx, format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<T>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        match **index {
            0 => Self::none(format, reader, cx),
            1 => Self::some(format, reader, cx),
            _ => Err(F::invalid_input_err()),
        }
    }

    fn none<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<T>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <() as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .map(|status| status.bimap(|_| Data::None, Self::None))
    }

    fn some<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<T>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <T as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .map(|status| status.bimap(Data::Some, Self::Some))
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

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Self::Data, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        Self::after_init(format, reader, cx)
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, <F as backend::Format>::Error>>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        let res = match self {
            Decode::Init => {
                Self::after_init(format, reader, cx)
            },
            Decode::Index(dec) => {
                futures::ready!(dec.poll_decode(format, reader, cx))
                .and_then(|idx| Self::after_index(&idx, format, reader, cx))
            }
            Decode::None(dec) => {
                futures::ready!(dec.poll_decode(format, reader, cx))
                .map(|_| backend::DecodeStatus::Ready(Data::None))
            }
            Decode::Some(dec) => {
                futures::ready!(dec.poll_decode(format, reader, cx))
                .map(|d| backend::DecodeStatus::Ready(Data::Some(d)))
            }
            Decode::Fini => {
                Err(F::invalid_input_err())
            }
        };

        match res {
            Ok(status) => {
                match status {
                    backend::DecodeStatus::Ready(d) => {
                        *self = Decode::Fini;
                        Poll::Ready(Ok(d))
                    }
                    backend::DecodeStatus::Pending(p) => {
                        *self = p;
                        Poll::Pending
                    }
                }
            },
            Err(e) => {
                *self = Decode::Fini;
                Poll::Ready(Err(e))
            }
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
        F: 'r + backend::FormatDeserialize<'r>,
        R: 'r + AsyncRead + AsyncBufRead + Unpin,
    = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: backend::FormatDeserialize<'r>,
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
    }
}