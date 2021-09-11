use core::marker::PhantomData;
use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead};
use crate::{backend, AsyncSerialize};

type Data<T> = ::std::cell::RefCell<T>;

wrapper_encode_def!();
wrapper_encodable_impl!();
wrapper_async_serialize_impl!();

wrapper_decode_def!();
wrapper_decode_impl!();
wrapper_decodable_impl!();
wrapper_async_deserialize_impl!();        


impl<F, T> backend::Encode for Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Format = F;
    type Data = Data<T>;

    fn init(data: &Self::Data) -> Self {
        Self(T::Encoder::<F>::init(&data.borrow()), PhantomData)
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <<Self as backend::Encode>::Format as backend::Format>::Error>
    where
        W: futures::AsyncWrite + Unpin,
    {
        T::Encoder::<F>::start_encode(format, writer, &data.borrow(), cx)
        .map(|o| o.map(|s| Self(s, PhantomData)))
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as backend::Encode>::Format as backend::Format>::Error>>
    where
        W: futures::AsyncWrite + Unpin,
    {
         self.0.poll_encode(format, writer, &data.borrow(), cx)
    }
}