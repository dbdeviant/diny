macro_rules! numeric_def {
    ($t:ty, $bytes:literal) => {
        use core::task::Context;
        use futures::{AsyncRead, AsyncBufRead, AsyncWrite};    
        use diny::{backend, buffer::{buffer_state::BufferState, BufferEncode}};
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
        
            fn start_encode_buffer<W>(_format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <<Self as BufferEncode>::Format as backend::Format>::Error>
            where
                W: AsyncWrite + Unpin,
            {
                let mut enc = Self::new(data);
                match enc.0.start_write(writer, cx) {
                    backend::PollEncodeStatus::Fini     => backend::StartEncodeStatus::Fini,
                    backend::PollEncodeStatus::Pending  => backend::StartEncodeStatus::Pending(enc),
                    backend::PollEncodeStatus::Error(e) => backend::StartEncodeStatus::Error(e),
                }
            }

            fn poll_encode_buffer<W>(&mut self, _format: &ThisFormat, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<Error>
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
        
            fn start_decode<R>(_format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <<Self as backend::Decode>::Format as backend::Format>::Error>
            where
                R: AsyncRead + AsyncBufRead + Unpin,
            {
                let mut decode = Self::init();
                match (&mut decode.0).start_read(reader, cx) {
                    backend::PollDecodeStatus::Fini(())    => backend::StartDecodeStatus::Fini(Data::from_le_bytes(*decode.0.buffer())),
                    backend::PollDecodeStatus::Pending    => backend::StartDecodeStatus::Pending(decode),
                    backend::PollDecodeStatus::Error(err) => backend::StartDecodeStatus::Error(err),
                }
            }

            fn poll_decode<R>(&mut self, _format: &ThisFormat, reader: &mut R, cx: &mut Context<'_>) -> diny::backend::PollDecodeStatus<Self::Data, Error>
            where
                R: AsyncRead + AsyncBufRead + Unpin,
            {
                match (&mut self.0).read_remaining(reader, cx) {
                    backend::PollDecodeStatus::Fini(())   => backend::PollDecodeStatus::Fini(Data::from_le_bytes(*self.0.buffer())),
                    backend::PollDecodeStatus::Pending    => backend::PollDecodeStatus::Pending,
                    backend::PollDecodeStatus::Error(err) => backend::PollDecodeStatus::Error(err),

                }
            }
        }

        serialize_all_def!    (ThisFormat, Data, Encoder);
        deserialize_exact_def!(ThisFormat, Data, Decoder);        
    };
}

macro_rules! usize_wrapper_def {
    ($t: ty, $repr: ty, $m: path) => {
        use core::{convert::TryInto, task::Context};
        use futures::{AsyncRead, AsyncBufRead, AsyncWrite};        
        use diny::backend::{self, Format};
        
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
        
            fn start_encode_buffer<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <<Self as diny::buffer::BufferEncode>::Format as backend::Format>::Error>
            where
                W: AsyncWrite + Unpin,
            {
                match TryInto::<$repr>::try_into(Into::<usize>::into(*data)) {
                    Ok(n) => {
                        match <wrapper::Encoder as diny::backend::Encode>::start_encode(format, writer, &n.into(), cx) {
                            backend::StartEncodeStatus::Fini         => backend::StartEncodeStatus::Fini,
                            backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self(Some(enc))),
                            backend::StartEncodeStatus::Error(e)     => backend::StartEncodeStatus::Error(e),
                        }
                    }
                    Err(_) => backend::StartEncodeStatus::Error(ThisFormat::invalid_data_err()),
                }
            }

            fn poll_encode_buffer<W>(&mut self, format: &ThisFormat, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<Error>
            where
                W: AsyncWrite + Unpin,
            {
                match &mut self.0 {
                    None      => backend::PollEncodeStatus::Error(ThisFormat::invalid_data_err()),
                    Some(enc) => enc.poll_encode_buffer(format, writer, cx),
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
        
            fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> diny::backend::StartDecodeStatus<Self::Data, Self, <<Self as diny::backend::Decode>::Format as Format>::Error>
            where
                R: AsyncRead + AsyncBufRead + Unpin,
            {
                match wrapper::Decoder::start_decode(format, reader, cx) {
                    diny::backend::StartDecodeStatus::Fini(n) => match TryInto::<usize>::try_into(n) {
                        Ok (n) => diny::backend::StartDecodeStatus::Fini(n.into()),
                        Err(_) => diny::backend::StartDecodeStatus::Error(ThisFormat::invalid_data_err()),
                    },
                    diny::backend::StartDecodeStatus::Pending(dec) => diny::backend::StartDecodeStatus::Pending(Decoder(dec)),
                    diny::backend::StartDecodeStatus::Error(e) => diny::backend::StartDecodeStatus::Error(e),
                }
            }

            fn poll_decode<R>(&mut self, format: &ThisFormat, reader: &mut R, cx: &mut Context<'_>) -> diny::backend::PollDecodeStatus<Self::Data, Error>
            where
                R: AsyncRead + AsyncBufRead + Unpin,
            {
                match self.0.poll_decode(format, reader, cx) {
                    diny::backend::PollDecodeStatus::Fini(n) => match TryInto::<usize>::try_into(n) {
                        Ok (n) => diny::backend::PollDecodeStatus::Fini(n.into()),
                        Err(_) => diny::backend::PollDecodeStatus::Error(ThisFormat::invalid_data_err()),
                    },
                    diny::backend::PollDecodeStatus::Pending => diny::backend::PollDecodeStatus::Pending,
                    diny::backend::PollDecodeStatus::Error(e) => diny::backend::PollDecodeStatus::Error(e),
                }
            }
        }
        
        serialize_all_def!    (ThisFormat, Data, Encoder);
        deserialize_exact_def!(ThisFormat, Data, Decoder);
    };
}