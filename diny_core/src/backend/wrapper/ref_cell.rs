use core::marker::PhantomData;
use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead};
use crate::{backend, AsyncSerialize};

type Data<T> = ::std::cell::RefCell<T>;

wrapper_encodable_impl!();
wrapper_async_serialize_impl!();

wrapper_decode_def!();
wrapper_decode_impl!();
wrapper_decodable_impl!();
wrapper_async_deserialize_impl!();        


pub struct Encode<F, T>(Option<T::Encoder::<F>>)
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
        match data.try_borrow() {
            Ok(ref d) => Self(Some(T::Encoder::<F>::init(d))),
            Err(_)    => Self(None),
        }
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <<Self as backend::Encode>::Format as backend::Format>::Error>
    where
        W: futures::AsyncWrite + Unpin,
    {
        match &data.try_borrow() {
            Ok(ref d) => 
                T::Encoder::<F>::start_encode(format, writer, d, cx)
                .map(|o| o.map(|s| Self(Some(s)))),
            Err(_) => Err(<Self as backend::Encode>::Format::invalid_input_err()),
        }
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as backend::Encode>::Format as backend::Format>::Error>>
    where
        W: futures::AsyncWrite + Unpin,
    {
        Poll::Ready(match &data.try_borrow() {
            Ok(ref d) => match &mut self.0 {
                Some(e) => futures::ready!(e.poll_encode(format, writer, d, cx)),
                None    => Err(<Self as backend::Encode>::Format::invalid_input_err()),
            },
            Err(_) => Err(<Self as backend::Encode>::Format::invalid_input_err()),
        })
    }
}