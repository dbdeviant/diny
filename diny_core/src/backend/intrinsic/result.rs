use core::task::{Context, Poll};
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

    fn after_init<W>(format: &F, writer: &mut W, data: &Data<O, E>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
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

    fn after_index<W>(format: &F, writer: &mut W, data: &Data<O, E>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        match data {
            Data::Ok (d) => Self::ok (format, writer, d, cx),
            Data::Err(e) => Self::err(format, writer, e, cx),
        }
    }

    fn ok<W>(format: &F, writer: &mut W, data: &O, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        <O as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map(|o| match o {
            None    => Self::Fini,
            Some(s) => Self::V0(s),
        })
    }

    fn err<W>(format: &F, writer: &mut W, data: &E, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        <E as backend::Encodable>::Encoder::<F>::start_encode(format, writer, data, cx)
        .map(|o| match o {
            None    => Self::Fini,
            Some(s) => Self::V1(s),
        })
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
                    Data::Ok(d) => futures::ready!(enc.poll_encode(format, writer, d, cx)).map(|_| Self::Fini),
                    _ => Err(F::invalid_input_err()),
                }
            }
            Self::V1(enc) => {
                match data {
                    Data::Err(e) => futures::ready!(enc.poll_encode(format, writer, e, cx)).map(|_| Self::Fini),
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


pub enum Decode<F, O, E>
where
    F: backend::FormatDecode,
    O: backend::Decodable,
    E: backend::Decodable,
{
    Init,
    Index(<VariantIdx as backend::Decodable>::Decoder<F>),
    None(<O as backend::Decodable>::Decoder<F>),
    Some(<E as backend::Decodable>::Decoder<F>),
    Fini,
}

impl<F, O, E> Decode<F, O, E>
where
    F: backend::FormatDecode,
    O: backend::Decodable,
    E: backend::Decodable,
{
    fn after_init<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<O, E>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <VariantIdx as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(|status| match status {
            backend::DecodeStatus::Ready(idx) => Self::after_index(&idx, format, reader, cx),
            backend::DecodeStatus::Pending(p) => Ok(backend::DecodeStatus::Pending(Self::Index(p))),
        })
    }

    fn after_index<R>(index: &VariantIdx, format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<O, E>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        match **index {
            0 => Self::ok (format, reader, cx),
            1 => Self::err(format, reader, cx),
            _ => Err(F::invalid_input_err()),
        }
    }

    fn ok<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<O, E>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <O as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .map(|status| status.bimap(Data::Ok, Self::None))
    }

    fn err<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Data<O, E>, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <E as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .map(|status| status.bimap(Data::Err, Self::Some))
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
                .map(|d| backend::DecodeStatus::Ready(Data::Ok(d)))
            }
            Decode::Some(dec) => {
                futures::ready!(dec.poll_decode(format, reader, cx))
                .map(|e| backend::DecodeStatus::Ready(Data::Err(e)))
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