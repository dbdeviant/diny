macro_rules! numeric_def {
    ($t:ty, $bytes:literal) => {
        use core::task::{Context, Poll};
        use futures::{AsyncRead, AsyncBufRead, AsyncWrite};    
        use diny::buffer::buffer_state::BufferState;
        use $crate::Formatter as ThisFormat;

        type Error = <ThisFormat as diny::backend::Format>::Error;

        type Data = $t;
        const BUF_SIZE: usize = $bytes;

        pub struct Encoder(BufferState<BUF_SIZE>);

        impl diny::buffer::BufferEncode for Encoder {
            type Data = Data;
            type Format = ThisFormat;

            fn new(data: &Self::Data) -> Self {
                Encoder(BufferState::with_contents(data.to_le_bytes()))
            }
        
            fn poll_encode_buffer<W>(&mut self, _format: &ThisFormat, writer: &mut W, cx: &mut Context<'_>) -> Poll<Result<(), Error>>
            where
                W: AsyncWrite + Unpin,
            {
                self.0.write_remaining(writer, cx)
            }
        }                

        pub struct Decoder(BufferState<BUF_SIZE>);

        impl diny::backend::Decode for Decoder {
            type Data = Data;
            type Format = ThisFormat;

            fn init() -> Self {
                Self(BufferState::init())
            }
        
            fn poll_decode<R>(&mut self, _format: &ThisFormat, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, Error>>
            where
                R: AsyncRead + AsyncBufRead + Unpin,
            {
                let this = &mut *self;
                futures::ready!(this.0.read_remaining(reader, cx))?;
                Poll::Ready(Ok(Data::from_le_bytes(*self.0.buffer())))
            }
        }

        serialize_all_def!    (ThisFormat, Data, Encoder);
        deserialize_exact_def!(ThisFormat, Data, Decoder);        
    };
}

macro_rules! usize_wrapper_def {
    ($t: ty, $repr: ty, $m: path) => {
        use core::{convert::TryInto, task::{Context, Poll}};
        use futures::{AsyncRead, AsyncBufRead, AsyncWrite};        
        use diny::backend::Format;
        
        use crate::{
            Formatter as ThisFormat,
            $m as wrapper,
        };
        
        pub type Data = $t;
        pub type Error = <ThisFormat as Format>::Error;
        
        pub struct Encoder(Option<wrapper::Encoder>);
        
        impl diny::buffer::BufferEncode for Encoder {
            type Data = Data;
            type Format = ThisFormat;
        
            fn new(data: &Self::Data) -> Self {
                Encoder(
                    TryInto::<$repr>::try_into(Into::<usize>::into(*data))
                    .map(|n| <wrapper::Encoder as diny::backend::Encode>::init(&n.into()))
                    .ok()
                )
            }
        
            fn poll_encode_buffer<W>(&mut self, format: &ThisFormat, writer: &mut W, cx: &mut Context<'_>) -> Poll<Result<(), Error>>
            where
                W: AsyncWrite + Unpin,
            {
                match &mut self.0 {
                    None    => Poll::Ready(Err(ThisFormat::invalid_data_err())),
                    Some(w) => w.poll_encode_buffer(format, writer, cx),
                }
            }
        }
        
        pub struct Decoder(wrapper::Decoder);
        
        impl diny::backend::Decode for Decoder {
            type Data = Data;
            type Format = ThisFormat;
        
            fn init() -> Self {
                Self(wrapper::Decoder::init())
            }
        
            fn poll_decode<R>(&mut self, format: &ThisFormat, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, Error>>
            where
                R: AsyncRead + AsyncBufRead + Unpin,
            {
                let n = futures::ready!(self.0.poll_decode(format, reader, cx))?;
                Poll::Ready(
                    match TryInto::<usize>::try_into(n) {
                        Ok (n) => Ok(n.into()),
                        Err(_) => Err(ThisFormat::invalid_data_err()),
                    }
                )
            }
        }
        
        serialize_all_def!    (ThisFormat, Data, Encoder);
        deserialize_exact_def!(ThisFormat, Data, Decoder);
    };
}