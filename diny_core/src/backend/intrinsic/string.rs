#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};
use crate::backend::{self, Encode as _, Decode as _, internal::SequenceLen};
use crate::buffer;


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
    fn after_init<W>(format: &F, writer: &mut W, data: &str, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        let len: SequenceLen = data.len().into();
        match <SequenceLen as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &len, cx) {
            Ok(o) => match o {
                None    => Self::after_len(format, writer, *len, data, cx),
                Some(s) => Ok(Self::Len(len, s)),
            }
            Err(e) => Err(e)
        }
    }

    fn after_len<W>(_format: &F, writer: &mut W, len: usize, data: &str, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        if len > 0 {
            let send = data.as_bytes();
            let mut cur = buffer::BufferCursor::new(send);
            match cur.write_remaining(writer, send, cx) {
                Poll::Ready(r) => r.map_or_else(|e| Err(e.into()), |()| Ok(Self::Fini)),
                Poll::Pending => Ok(Self::Cur(cur)),
            }
        } else {
            Ok(Self::Fini)
        }
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

    fn start_encode<W>(format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Result<Option<Self>, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        Self::after_init(format, writer, data, cx)
        .map(|s| match s {
            Self::Fini => None,
            _          => Some(s),
        })
    }

    fn poll_encode<W>(&mut self, format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> Poll<Result<(), <<Self as backend::Encode>::Format as backend::Format>::Error>>
    where
        W: AsyncWrite + Unpin,
    {
        let res = match self {
            Self::Init => {
                Self::after_init(format, writer, data, cx)
            },
            Self::Len(len, enc) => {
                futures::ready!(enc.poll_encode(format, writer, len, cx))
                .and_then(|_| Self::after_len(format, writer, **len, data, cx))
            }
            Self::Cur(cur) => {
                futures::ready!(cur.write_remaining(writer, data.as_bytes(), cx))
                .map_err(|e| e.into())
                .map(|_| Self::Fini)
            }
            Self::Fini => {
                Err(F::invalid_input_err())
            }
        };

        match res {
            Ok(enc) => {
                *self = enc;
                match self {
                    Self::Fini => Poll::Ready(Ok(())),
                    _          => Poll::Pending,
                }
            },
            Err(e) => {
                *self = Self::Fini;
                Poll::Ready(Err(e))
            }
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
        F: 'w + backend::FormatSerialize<'w>,
        W: 'w + AsyncWrite + Unpin,
    = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        Self: 'w,
        F: backend::FormatSerialize<'w>,
        W: AsyncWrite + Unpin,

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
    fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <SequenceLen as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(|s| match s {
            backend::DecodeStatus::Ready  (len) => Self::after_len(format, reader, *len, data, cx),
            backend::DecodeStatus::Pending(dec) => Ok(Self::Len(dec))
        })
    }

    fn after_len<R>(_format: &F, reader: &mut R, len: usize, data: &mut PartialData, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        if len > 0 {
            data.reserve_exact(len);
            let mut cur = buffer::BufferCursor::with_len(len);
            match cur.fill_vec(reader, data, cx) {
                Poll::Ready(r) => r.map_or_else(|e| Err(e.into()), |()| Ok(Self::Fini)),
                Poll::Pending => Ok(Self::Cur(cur)),
            }
        } else {
            Ok(Self::Fini)
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

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> Result<backend::DecodeStatus<Self::Data, Self>, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        let mut data = PartialData::new();
        DecodeCursor::after_init(format, reader, &mut data, cx)
        .and_then(|cursor| match cursor {
            DecodeCursor::Fini => data
                                    .into_data::<F>()
                                    .map(backend::DecodeStatus::Ready),
            _                  => Ok(backend::DecodeStatus::Pending(Self { state: Some(DecodeState { data, cursor }) }))
        })
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, <F as backend::Format>::Error>>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        struct RetData;

        let _ret_data = match &mut self.state {
            None => return Poll::Ready(Err(F::invalid_input_err())),
            Some(state) => {
                let data = &mut state.data;
                let res = match &mut state.cursor {
                    DecodeCursor::Init => {
                        DecodeCursor::after_init(format, reader, &mut state.data, cx)
                    }
                    DecodeCursor::Len(dec) => {
                        futures::ready!(dec.poll_decode(format, reader, cx))
                        .and_then(|len| DecodeCursor::after_len(format, reader, *len, data, cx))
                    }
                    DecodeCursor::Cur(cur) => {
                        futures::ready!(cur.fill_vec(reader, data, cx))
                        .map_or_else(
                            |e | Err(e.into()),
                            |()| Ok(DecodeCursor::Fini),
                        )
                    }
                    DecodeCursor::Fini => {
                        Err(F::invalid_input_err())
                    }
                };

                match res {
                    Ok(dec) => {
                        state.cursor = dec;
                        match state.cursor {
                            DecodeCursor::Fini => RetData, // the one and only statement value
                            _ => return Poll::Pending,
                        }
                    }
                    Err(e) => {
                        state.cursor = DecodeCursor::Fini;
                        return Poll::Ready(Err(e))
                    },
                };
            }
        };

        // SAFETY:
        // The only way this code gets executed is if the state existed and reached
        // the DecodeCursor::Fini state.  That cursor state is only reached once all
        // string bytes have been read, and this next statement consumes the outer
        // state in order to produce the returned string
        Poll::Ready(self.state.take().unwrap().data.into_data::<F>())
    }
}

impl backend::Decodable for Data {
    type Decoder<F: backend::FormatDecode> = Decode<F>;
}

impl backend::AsyncDeserialize for Data {
    type Future<'r, F, R>
    where
        F: 'r + backend::FormatDeserialize<'r>,
        R: 'r + AsyncRead + AsyncBufRead + Unpin,
    = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: backend::FormatDeserialize<'r>,
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
    }
}