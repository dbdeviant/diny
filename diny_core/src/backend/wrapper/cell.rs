use core::marker::PhantomData;
use core::task::Context;
use crate::{backend, AsyncSerialize, io};

type Data<T> = ::std::cell::Cell<T>;

wrapper_encode_def!();

wrapper_decode_def!();
wrapper_decode_impl!();
wrapper_decodable_impl!();
wrapper_async_deserialize_impl!();        


impl<F, T> backend::Encode for Encoder<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable + Copy,
{
    type Format = F;
    type Data = Data<T>;

    fn init(data: &Self::Data) -> Self {
        Self(T::Encoder::<F>::init(&data.get()))
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        T::Encoder::<F>::start_encode(format, writer, &data.get(), cx)
        .map_pending(Self)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
         self.0.poll_encode(format, writer, &data.get(), cx)
    }
}

impl<T> backend::Encodable for Data<T>
where
    T: backend::Encodable + Copy,
{
    type Encoder<F>
    where
        F: backend::FormatEncode,
    = Encoder<F, T>;
}

impl<'t, T> AsyncSerialize for Data<T>
where
    T: backend::Encodable + Copy,
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
