use core::marker::PhantomData;
use core::task::{Context, Poll};

use futures::{AsyncRead, AsyncBufRead};

use crate::{backend, AsyncSerialize};

type Data<T> = ::std::sync::Arc<T>;

pub struct Encode<F, T>(T::Encoder::<F>, PhantomData<F>)
where
    F: backend::FormatEncode,
    T: backend::Encodable,
;

impl<F, T> backend::Encode for Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Format = F;
    type Data = Data<T>;

    fn init(data: &Self::Data) -> Self {
        Self(T::Encoder::<F>::init(data), PhantomData)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <<Self as backend::Encode>::Format as backend::Format>::Error>
    where
        W: futures::AsyncWrite + Unpin,
    {
        T::Encoder::<F>::start_encode(format, writer, data, cx)
        .map(|o| o.map(|s| Self(s, PhantomData)))
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as backend::Encode>::Format as backend::Format>::Error>>
    where
        W: futures::AsyncWrite + Unpin,
    {
         self.0.poll_encode(format, writer, data, cx)
    }
}

impl<T> backend::Encodable for Data<T>
where
    T: backend::Encodable,
{
    type Encoder<F>
    where
        F: backend::FormatEncode,
    = Encode<F, T>;
}

impl<'t, T> AsyncSerialize for Data<T>
where
    T: backend::Encodable,
{
    type Future<'w, F, W>
    where
        Self: 'w,
        F: 'w + crate::backend::FormatSerialize<'w>,
        W: 'w + ::futures::AsyncWrite + Unpin,
    = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        F: crate::backend::FormatSerialize<'w>,
        W: ::futures::AsyncWrite + Unpin,
    {
        backend::SerializeAll::new(format, writer, self, <Self::Encoder::<F> as backend::Encode>::init(self))
    }
}


pub struct Decode<F, T>(T::Decoder::<F>, PhantomData<F>)
where
    F: backend::FormatDecode,
    T: backend::Decodable,
;

impl<F, T> backend::Decode for Decode<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    type Format = F;
    type Data = Data<T>;

    fn init() -> Self {
        Self(T::Decoder::<F>::init(), PhantomData)
    }

    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Self::Data, Self>, <<Self as backend::Decode>::Format as backend::Format>::Error>
    where
        R: futures::AsyncRead + AsyncBufRead + Unpin,
    {
        T::Decoder::<F>::start_decode(format, reader, cx)
        .map(|o| o.bimap(
            ::std::sync::Arc::new,
            |s| Self(s, PhantomData),
        ))
    }

    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, <<Self as backend::Decode>::Format as backend::Format>::Error>>
    where
        R: futures::AsyncRead + AsyncBufRead + Unpin,
     {
        let d = futures::ready!(self.0.poll_decode(format, reader, cx))?;
        Poll::Ready(Ok(::std::sync::Arc::new(d)))
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