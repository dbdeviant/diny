#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};
use crate::backend::{self, Encode as _, Decode as _, internal::SequenceLen};


type Data<T> = Vec<T>;

type Len = usize;
type Idx = usize;

pub enum Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    Init,
    Len(SequenceLen, <SequenceLen as backend::Encodable>::Encoder<F>),
    Cur(Len, Idx, <T as backend::Encodable>::Encoder<F>),
    Fini,
}

impl<F, T> Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    fn after_init<W>(format: &F, writer: &mut W, data: &[T], cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        let len: SequenceLen = data.len().into();
        match <SequenceLen as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &len, cx) {
            Ok(o) => match o {
                None => Self::after_len(format, writer, *len, data, cx),
                Some(s) => Ok(Self::Len(len, s)),
            }
            Err(e) => Err(e)
        }
    }

    fn after_len<W>(format: &F, writer: &mut W, len: Len, data: &[T], cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        Self::items_from(format, writer, len, 0, data, cx)
    }
        
    fn items_from<W>(format: &F, writer: &mut W, len: Len, idx: Idx, data: &[T], cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        for (i, d) in data.iter().enumerate().take(len).skip(idx) {
            match <T as backend::Encodable>::Encoder::<F>::start_encode(format, writer, d, cx) {
                Ok(o) => match o {
                    None    => continue,
                    Some(s) => return Ok(Self::Cur(len, i, s)),
                }
                Err(e) => return Err(e)
            }
        }

        Ok(Self::Fini)
    }
}

impl<F, T> backend::Encode for Encode<F, T>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Data = Data<T>;
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
            Self::Cur(len, idx, enc) => {
                futures::ready!(enc.poll_encode(format, writer, &data[*idx], cx))
                .and_then(|_| Self::items_from(format, writer, *len, *idx + 1, data, cx))
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

impl<T> backend::Encodable for Data<T>
where
    T: backend::Encodable,
{
    type Encoder<F: backend::FormatEncode> = Encode<F, T>;
}

impl<T> backend::AsyncSerialize for Data<T>
where
    T: backend::Encodable,
{
    type Future<'w, F, W>
    where
        Self: 'w,
        F: 'w + backend::FormatSerialize,
        W: 'w + AsyncWrite + Unpin,
    = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        F: backend::FormatSerialize,
        W: AsyncWrite + Unpin,

    {
        backend::SerializeAll::new(format, writer, self, <Self::Encoder::<F> as backend::Encode>::init(self))
    }
}

struct PartialData<T>(Vec<T>);

impl<T> PartialData<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn into_data(self) -> Vec<T> {
        self.0
    }
}

impl<T> core::ops::Deref for PartialData<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> core::ops::DerefMut for PartialData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

enum DecodeCursor<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    Init,
    Len(<SequenceLen as backend::Decodable>::Decoder<F>),
    Cur(Len, Idx, <T as backend::Decodable>::Decoder<F>),
    Fini,
}

struct DecodeState<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    data: PartialData<T>,
    cursor: DecodeCursor<F, T>,
}

impl<F, T> DecodeState<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    pub fn new() -> Self {
        Self {
            data: PartialData::new(),
            cursor: DecodeCursor::Init,
        }
    }
}

impl<F, T> DecodeCursor<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData<T>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        <SequenceLen as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(|s| match s {
            backend::DecodeStatus::Ready  (len) => Self::after_len(format, reader, *len, data, cx),
            backend::DecodeStatus::Pending(dec) => Ok(Self::Len(dec))
        })
    }

    fn after_len<R>(format: &F, reader: &mut R, len: Len, data: &mut PartialData<T>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        data.reserve_exact(len);
        Self::items_from(format, reader, len, 0, data, cx)
    }

    fn items_from<R>(format: &F, reader: &mut R, len: Len, idx: Idx, data: &mut PartialData<T>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        for i in idx..len {
            match <T as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                Ok(status) => match status {
                    backend::DecodeStatus::Ready(d) => { data.push(d); continue },
                    backend::DecodeStatus::Pending(p) => return Ok(Self::Cur(len, i, p)),
                },
                Err(e) => return Err(e),
            }
        }

        Ok(Self::Fini)
    }
}

pub struct Decode<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    state: Option<DecodeState<F, T>>,
}

impl<F, T> backend::Decode for Decode<F, T>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    type Data = Data<T>;
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
        .map(|cursor| match cursor {
            DecodeCursor::Fini => backend::DecodeStatus::Ready(data.into_data()),
            _                  => backend::DecodeStatus::Pending(Self { state: Some(DecodeState { data, cursor }) })
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
                    DecodeCursor::Cur(len, idx, dec) => {
                        futures::ready!(dec.poll_decode(format, reader, cx))
                        .and_then(|d| {
                            data.push(d);
                            DecodeCursor::items_from(format, reader, *len, *idx + 1, data, cx)
                        })
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
        Poll::Ready(Ok(self.state.take().unwrap().data.into_data()))
    }
}

impl<T> backend::Decodable for Data<T>
where
    T: backend::Decodable,
{
    type Decoder<F: backend::FormatDecode> = Decode<F, T>;
}

impl<T> backend::AsyncDeserialize for Data<T>
where
    T: backend::Decodable,
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