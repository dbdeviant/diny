use core::marker::PhantomData;
use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead};
use crate::{backend, AsyncSerialize};

type Data<T> = PhantomData<T>;

wrapper_encodable_impl!();
wrapper_async_serialize_impl!();

wrapper_decodable_impl!();
wrapper_async_deserialize_impl!();

pub struct Encode<F, T>(<() as backend::Encodable>::Encoder::<F>, PhantomData<T>)
where
    F: backend::FormatEncode,
;

impl<F, T> backend::Encode for Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Format = F;
    type Data = Data<T>;

    fn init(_data: &Self::Data) -> Self {
        Self(<() as backend::Encodable>::Encoder::<F>::init(&()), PhantomData)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <<Self as backend::Encode>::Format as backend::Format>::Error>
    where
        W: futures::AsyncWrite + Unpin,
    {
        <() as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &(), cx)
        .map(|o| o.map(|s| Self(s, PhantomData)))
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as backend::Encode>::Format as backend::Format>::Error>>
    where
        W: futures::AsyncWrite + Unpin,
    {
            self.0.poll_encode(format, writer, &(), cx)
    }
}

pub struct Decode<F, T>(<() as backend::Decodable>::Decoder::<F>, PhantomData<T>)
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
        Self(<() as backend::Decodable>::Decoder::<F>::init(), PhantomData)
    }

    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Self::Data, Self>, <<Self as backend::Decode>::Format as backend::Format>::Error>
    where
        R: futures::AsyncRead + AsyncBufRead + Unpin,
    {
        <() as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .map(|o| o.bimap(
            |_| PhantomData,
            |s| Self(s, PhantomData),
        ))
    }

    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, <<Self as backend::Decode>::Format as backend::Format>::Error>>
    where
        R: futures::AsyncRead + AsyncBufRead + Unpin,
        {
        let _ = futures::ready!(self.0.poll_decode(format, reader, cx))?;
        Poll::Ready(Ok(PhantomData))
    }
}