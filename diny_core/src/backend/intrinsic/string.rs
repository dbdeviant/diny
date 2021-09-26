#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::task::Context;
use crate::backend::{self, Encode as _, Decode as _, internal::SequenceLen};
use crate::{buffer, io};


type Data = String;

pub enum Encode<F>
where
    F: backend::FormatEncode,
{
    Init,
    Len(SequenceLen, <SequenceLen as backend::Encodable>::Encoder<F>),
    Cur(buffer::BufferCursor),
    Fini,
}

impl<F> Encode<F>
where
    F: backend::FormatEncode,
{
    fn after_init<W>(format: &F, writer: &mut W, data: &str, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        let len: SequenceLen = data.len().into();
        match <SequenceLen as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &len, cx) {
            backend::StartEncodeStatus::Fini         => Self::after_len(format, writer, *len, data, cx),
            backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self::Len(len, enc)),
            backend::StartEncodeStatus::Error(e)     => backend::StartEncodeStatus::Error(e)
        }
    }

    fn after_len<W>(_format: &F, writer: &mut W, len: usize, data: &str, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        if len > 0 {
            let send = data.as_bytes();
            let mut cur = buffer::BufferCursor::new(send);
            match cur.write_remaining(writer, send, cx) {
                backend::PollEncodeStatus::Fini     => backend::StartEncodeStatus::Fini,
                backend::PollEncodeStatus::Pending  => backend::StartEncodeStatus::Pending(Self::Cur(cur)),
                backend::PollEncodeStatus::Error(e) => backend::StartEncodeStatus::Error(e.into()),
            }
        } else {
            backend::StartEncodeStatus::Fini
        }
    }

    fn poll_cur<W>(_format: &F, writer: &mut W, cur: &mut buffer::BufferCursor, data: &str, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        cur.write_remaining(writer, data.as_bytes(), cx)
        .map_err(|e| e.into())
    }
}

impl<F> backend::Encode for Encode<F>
where
    F: backend::FormatEncode,
{
    type Data = Data;
    type Format = F;

    fn init(_data: &Self::Data) -> Self {
        Self::Init
    }

    fn start_encode<W>(format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        Self::after_init(format, writer, data, cx)
    }

    fn poll_encode<W>(&mut self, format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        match self {
            Self::Init          => encode_chain!(*self, Self::start_encode(format, writer, data, cx)),
            Self::Len(len, enc) => encode_poll_chain!(*self, enc.poll_encode(format, writer, len, cx), Self::after_len(format, writer, **len, data, cx)),
            Self::Cur(cur)      => encode_poll_fini!(*self, Self::poll_cur(format, writer, cur, data, cx)),
            Self::Fini          => backend::PollEncodeStatus::Error(F::invalid_input_err()),
        }
    }
}

impl backend::Encodable for Data {
    type Encoder<F: backend::FormatEncode> = Encode<F>;
}


impl backend::AsyncSerialize for Data {
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

struct PartialData(Vec<u8>);

impl PartialData {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn into_data<F>(self) -> Result<String, F::Error>
    where
        F: backend::Format,
    {
        String::from_utf8(self.0).map_err(|_| F::invalid_data_err())
    }
}

impl core::ops::Deref for PartialData {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for PartialData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

enum DecodeCursor<F>
where
    F: backend::FormatDecode,
{
    Init,
    Len(<SequenceLen as backend::Decodable>::Decoder<F>),
    Cur(buffer::BufferCursor),
    Fini,
}

struct DecodeState<F>
where
    F: backend::FormatDecode,
{
    data: PartialData,
    cursor: DecodeCursor<F>,
}

impl<F> DecodeState<F>
where
    F: backend::FormatDecode,
{
    pub fn new() -> Self {
        Self {
            data: PartialData::new(),
            cursor: DecodeCursor::Init,
        }
    }
}

impl<F> DecodeCursor<F>
where
    F: backend::FormatDecode,
{
    fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        <SequenceLen as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |len| Self::after_len(format, reader, *len, data, cx),
            Self::Len,
        )
    }

    fn after_len<R>(_format: &F, reader: &mut R, len: usize, data: &mut PartialData, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        if len > 0 {
            data.reserve_exact(len);
            let mut cur = buffer::BufferCursor::with_len(len);
            match cur.fill_vec(reader, data, cx) {
                backend::PollDecodeStatus::Fini(()) => backend::StartDecodeStatus::Fini(()),
                backend::PollDecodeStatus::Pending  => backend::StartDecodeStatus::Pending(Self::Cur(cur)),
                backend::PollDecodeStatus::Error(e) => backend::StartDecodeStatus::Error(e.into()),
            }
        } else {
            backend::StartDecodeStatus::Fini(())
        }
    }
}

pub struct Decode<F>
where
    F: backend::FormatDecode,
{
    state: Option<DecodeState<F>>,
}

impl<F> backend::Decode for Decode<F>
where
    F: backend::FormatDecode,
{
    type Data = Data;
    type Format = F;

    fn init() -> Self {
        Self { state: Some(DecodeState::new()) }
    }

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        let mut data = PartialData::new();
        match DecodeCursor::after_init(format, reader, &mut data, cx) {
            backend::StartDecodeStatus::Fini(())        => data.into_data::<F>().into(),
            backend::StartDecodeStatus::Pending(cursor) => backend::StartDecodeStatus::Pending(Self { state: Some(DecodeState { data, cursor }) }),
            backend::StartDecodeStatus::Error(e)        => backend::StartDecodeStatus::Error(e),
        }
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
    where
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        if let Some(state) = &mut self.state {
            match &mut state.cursor {
                DecodeCursor::Init => decode_chain!(state.cursor, DecodeCursor, DecodeCursor::after_init(format, reader, &mut state.data, cx)),
                DecodeCursor::Len(dec) =>
                    decode_poll_chain!(
                        state.cursor,
                        DecodeCursor,
                        dec.poll_decode(format, reader, cx),
                        |len: SequenceLen| {
                            DecodeCursor::after_len(format, reader, *len, &mut state.data, cx)
                        }
                    ),
                DecodeCursor::Cur(cur) =>
                    decode_poll_fini!(
                        state.cursor,
                        DecodeCursor,
                        cur.fill_vec(reader, &mut state.data, cx).map_err(|e| e.into()),
                        |()| ()
                    ),
                DecodeCursor::Fini => return backend::PollDecodeStatus::Error(F::invalid_input_err()),
            }
            // SAFETY:
            // The only way this code gets executed is if the outer state existed and reached
            // the DecodeCursor::Fini state as a result of this call.  That cursor state is
            // only reached once all array items have been created, and this next statement
            // consumes the outer state in order to produce the returned array.
            .and_then(|()| self.state.take().unwrap().data.into_data::<F>().into())
        } else {
            backend::PollDecodeStatus::Error(F::invalid_input_err())
        }
    }
}

impl backend::Decodable for Data {
    type Decoder<F: backend::FormatDecode> = Decode<F>;
}

impl backend::AsyncDeserialize for Data {
    type Future<'r, F, R>
    where
        F: 'r + backend::FormatDeserialize,
        R: 'r + io::AsyncRead + io::AsyncBufRead + Unpin,
    = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: backend::FormatDeserialize,
        R: io::AsyncRead + io::AsyncBufRead + Unpin,
    {
        backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
    }
}