use core::marker::PhantomData;
use core::task::Context;
use crate::{backend, AsyncSerialize, io};

type Data<T> = PhantomData<T>;

wrapper_encodable_impl!();
wrapper_async_serialize_impl!();

wrapper_decodable_impl!();
wrapper_async_deserialize_impl!();

pub struct Encoder<F, T>(<() as backend::Encodable>::Encoder::<F>, PhantomData<T>)
where
    F: backend::FormatEncode,
;

impl<F, T> backend::Encode for Encoder<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Format = F;
    type Data = Data<T>;

    fn init(_data: &Self::Data) -> Self {
        Self(<() as backend::Encodable>::Encoder::<F>::init(&()), PhantomData)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        <() as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &(), cx)
        .map_pending(|s| Self(s, PhantomData))
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, _data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.0.poll_encode(format, writer, &(), cx)
    }
}

pub struct Decoder<F, T>(<() as backend::Decodable>::Decoder::<F>, PhantomData<T>)
where
    F: backend::FormatDecode,
    T: backend::Decodable,
;

impl<F, T> backend::Decode for Decoder<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    type Format = F;
    type Data = Data<T>;

    fn init() -> Self {
        Self(<() as backend::Decodable>::Decoder::<F>::init(), PhantomData)
    }

    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>)
        -> backend::StartDecodeStatus<Self::Data, Self, <<Self as backend::Decode>::Format as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        <() as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .bimap(
            |()| PhantomData,
            |s| Self(s, PhantomData),
        )
    }

    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) 
        -> backend::PollDecodeStatus<Self::Data, <<Self as backend::Decode>::Format as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        self.0.poll_decode(format, reader, cx)
        .map(|()| PhantomData)
    }
}