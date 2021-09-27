macro_rules! numeric_encode_decode_def {
    () => {
        pub struct Encoder(BufferState<BUF_SIZE>);

        impl diny::buffer::BufferEncode for Encoder {
            type Data = Data;
            type Format = ThisFormat;

            fn init_buffer(data: &Self::Data) -> Self {
                Encoder(BufferState::with_contents(to_le_bytes(*data)))
            }
        
            fn start_encode_buffer<W>(_format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                let mut enc = Self::init_buffer(data);
                enc.0.start_write(writer, cx)
                .lift(enc)
            }

            fn poll_encode_buffer<W>(&mut self, _format: &Self::Format, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                self.0.write_remaining(writer, cx)
            }
        }                

        pub struct Decoder(BufferState<BUF_SIZE>);

        impl backend::Decode for Decoder {
            type Data = Data;
            type Format = ThisFormat;

            fn init() -> Self {
                Self(BufferState::init())
            }
        
            fn start_decode<R>(_format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                let mut dec = Self::init();
                match (&mut dec.0).start_read(reader, cx) {
                    backend::PollDecodeStatus::Fini(())   => from_le_bytes(*dec.0.buffer()).into(),
                    backend::PollDecodeStatus::Pending    => backend::StartDecodeStatus::Pending(dec),
                    backend::PollDecodeStatus::Error(err) => backend::StartDecodeStatus::Error(err),
                }
            }

            fn poll_decode<R>(&mut self, _format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                match (&mut self.0).read_remaining(reader, cx) {
                    backend::PollDecodeStatus::Fini(())   => from_le_bytes(*self.0.buffer()).into(),
                    backend::PollDecodeStatus::Pending    => backend::PollDecodeStatus::Pending,
                    backend::PollDecodeStatus::Error(err) => backend::PollDecodeStatus::Error(err),
                }
            }
        }
    };
}

macro_rules! numeric_def {
    ($t:ty, $bytes:literal) => {
        use core::task::Context;
        use diny::{backend, buffer::{buffer_state::BufferState}, io};
        use $crate::Formatter as ThisFormat;

        type Error = <ThisFormat as backend::Format>::Error;
        type Data = $t;
        const BUF_SIZE: usize = $bytes;

        #[inline(always)]
        fn to_le_bytes(v: Data) -> [u8; BUF_SIZE] {
            v.to_le_bytes()
        }
        
        #[inline(always)]
        fn from_le_bytes(bytes: [u8; BUF_SIZE]) -> Data {
            Data::from_le_bytes(bytes)
        }
        

        numeric_encode_decode_def!();
        serialize_all_def!    (ThisFormat, Data, Encoder);
        deserialize_exact_def!(ThisFormat, Data, Decoder);        
    };
}

macro_rules! usize_wrapper_def {
    ($t: ty, $repr: ty, $m: path) => {
        use core::{convert::TryInto, task::Context};
        use diny::{backend::{self, Format}, buffer, io};
        
        use crate::{
            Formatter as ThisFormat,
            $m as wrapper,
        };
        
        pub type Data = $t;
        pub type Error = <ThisFormat as Format>::Error;
        
        pub struct Encoder(Option<wrapper::Encoder>);
        
        impl buffer::BufferEncode for Encoder {
            type Data = Data;
            type Format = ThisFormat;
        
            fn init_buffer(data: &Self::Data) -> Self {
                Encoder(
                    TryInto::<$repr>::try_into(Into::<usize>::into(*data))
                    .map(|n| <wrapper::Encoder as diny::backend::Encode>::init(&n.into()))
                    .ok()
                )
            }
        
            fn start_encode_buffer<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <<Self as diny::buffer::BufferEncode>::Format as backend::Format>::Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                match TryInto::<$repr>::try_into(Into::<usize>::into(*data)) {
                    Ok(n) => {
                        <wrapper::Encoder as diny::backend::Encode>::start_encode(format, writer, &n.into(), cx)
                        .map_pending(|enc| Self(Some(enc)))
                    }
                    Err(_) => backend::StartEncodeStatus::Error(Self::Format::invalid_data_err()),
                }
            }

            fn poll_encode_buffer<W>(&mut self, format: &Self::Format, writer: &mut W, cx: &mut Context<'_>) -> backend::PollEncodeStatus<Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                match &mut self.0 {
                    None      => backend::PollEncodeStatus::Error(Self::Format::invalid_data_err()),
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
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                wrapper::Decoder::start_decode(format, reader, cx)
                .and_then(
                    |n| TryInto::<usize>::try_into(n)
                        .map(|n| n.into())
                        .map_err(|_| Self::Format::invalid_data_err())
                        .into(),
                    Decoder,
                )
            }

            fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> diny::backend::PollDecodeStatus<Self::Data, Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                self.0.poll_decode(format, reader, cx)
                .and_then(
                    |n| TryInto::<usize>::try_into(n)
                        .map(|n| n.into())
                        .map_err(|_| Self::Format::invalid_data_err())
                        .into()                    
                )
            }
        }
        
        serialize_all_def!    (ThisFormat, Data, Encoder);
        deserialize_exact_def!(ThisFormat, Data, Decoder);
    };
}