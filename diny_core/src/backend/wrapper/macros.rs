
macro_rules! wrapper_encode_def {
    () => {
        pub struct Encode<F, T>(T::Encoder::<F>)
        where
            F: backend::FormatEncode,
            T: backend::Encodable,
        ;
    };
}

macro_rules! wrapper_encode_impl_deref {
    () => {
        impl<F, T> backend::Encode for Encode<F, T>
        where
            F: backend::FormatEncode,
            T: backend::Encodable,
        {
            type Format = F;
            type Data = Data<T>;
        
            fn init(data: &Self::Data) -> Self {
                Self(T::Encoder::<F>::init(data))
            }
        
            fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                W: futures::AsyncWrite + Unpin,
            {
                T::Encoder::<F>::start_encode(format, writer, data, cx)
                .map_pending(Self)
            }
        
            fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
            where
                W: futures::AsyncWrite + Unpin,
            {
                 self.0.poll_encode(format, writer, data, cx)
            }
        }
    };
}

macro_rules! wrapper_encodable_impl {
    () => {
        impl<T> backend::Encodable for Data<T>
        where
            T: backend::Encodable,
        {
            type Encoder<F>
            where
                F: backend::FormatEncode,
            = Encode<F, T>;
        }
    };
}

macro_rules! wrapper_async_serialize_impl {
    () => {
        impl<'t, T> AsyncSerialize for Data<T>
        where
            T: backend::Encodable,
        {
            type Future<'w, F, W>
            where
                Self: 'w,
                F: 'w + crate::backend::FormatSerialize,
                W: 'w + ::futures::AsyncWrite + Unpin,
            = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;
        
            fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
            where
                F: crate::backend::FormatSerialize,
                W: ::futures::AsyncWrite + Unpin,
            {
                backend::SerializeAll::new(format, writer, self, <Self::Encoder::<F> as backend::Encode>::init(self))
            }
        }
    };
}

macro_rules! wrapper_decode_def {
    () => {
        pub struct Decode<F, T>(T::Decoder::<F>, PhantomData<F>)
        where
            F: backend::FormatDecode,
            T: backend::Decodable,
        ;
    };
}

macro_rules! wrapper_decode_impl {
    () => {
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
        
            fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
            where
                R: futures::AsyncRead + AsyncBufRead + Unpin,
            {
                T::Decoder::<F>::start_decode(format, reader, cx)
                .bimap(
                    Data::<T>::new,
                    |s| Self(s, PhantomData),
                )
            }
        
            fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
            where
                R: futures::AsyncRead + AsyncBufRead + Unpin,
             {
                self.0.poll_decode(format, reader, cx)
                .map(Data::<T>::new)
            }
        }
    }
}

macro_rules! wrapper_decodable_impl {
    () => {
        impl<T> backend::Decodable for Data<T>
        where
            T: backend::Decodable,
        {
            type Decoder<F: backend::FormatDecode> = Decode<F, T>;
        }
    };
}

macro_rules! wrapper_async_deserialize_impl {
    () => {
        impl<T> backend::AsyncDeserialize for Data<T>
        where
            T: backend::AsyncDeserialize,
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
    };
}


macro_rules! wrapper_deref {
    ($t: ty) => {
        use core::marker::PhantomData;
        use core::task::Context;
        use futures::{AsyncRead, AsyncBufRead};
        use crate::{backend, AsyncSerialize};

        type Data<T> = $t;

        wrapper_encode_def!();
        wrapper_encode_impl_deref!();
        wrapper_encodable_impl!();
        wrapper_async_serialize_impl!();

        wrapper_decode_def!();
        wrapper_decode_impl!();
        wrapper_decodable_impl!();
        wrapper_async_deserialize_impl!();        
    }
}

