use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};
use crate::backend::{self, Encode as _, Decode as _};


type Data<T, const L: usize> = [T; L];

pub enum Encode<F, T, const L: usize>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    Init,
    Cur(usize, <T as backend::Encodable>::Encoder<F>),
    Fini,
}

impl<F, T, const L: usize> Encode<F, T, L>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    fn after_init<W>(format: &F, writer: &mut W, data: &Data<T, L>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        Self::fields_from(format, writer, 0, data, cx)
    }

    fn fields_from<W>(format: &F, writer: &mut W, idx: usize, data: &Data<T, L>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        W: AsyncWrite + Unpin,
    {
        for (i, d) in data.iter().enumerate().take(L).skip(idx) {
            match <T as backend::Encodable>::Encoder::<F>::start_encode(format, writer, d, cx) {
                Ok(o) => match o {
                    None    => continue,
                    Some(s) => return Ok(Self::Cur(i, s)),
                }
                Err(e) => return Err(e)
            }
        }

        Ok(Self::Fini)
    }
}

impl<F, T, const L: usize> backend::Encode for Encode<F, T, L>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Data = Data<T, L>;
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
            Self::Cur(idx, enc) => {
                futures::ready!(enc.poll_encode(format, writer, &data[*idx], cx))
                .and_then(|_| Self::fields_from(format, writer, *idx + 1, data, cx))
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

impl<T, const L: usize> backend::Encodable for Data<T, L>
where
    T: backend::Encodable,
{
    type Encoder<F: backend::FormatEncode> = Encode<F, T, L>;
}


impl<T, const L: usize> backend::AsyncSerialize for Data<T, L>
where
    T: backend::AsyncSerialize,
{
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

struct PartialData<T, const L: usize>([Option<T>; L]);

impl<T, const L: usize> PartialData<T, L> {
    fn new() -> Self {
        Self([(); L].map(|()| None))
    }
    
    fn into_data(self) -> [T; L] {
        self.0.map(|o| o.unwrap())
    }
}

impl<T, const L: usize> core::ops::Deref for PartialData<T, L> {
    type Target = [Option<T>; L];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const L: usize> core::ops::DerefMut for PartialData<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


enum DecodeCursor<F, T, const L: usize>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    Init,
    Cur(usize, <T as backend::Decodable>::Decoder<F>),
    Fini,
}

struct DecodeState<F, T, const L: usize>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    data: PartialData<T, L>,
    cursor: DecodeCursor<F, T, L>,
}

impl<F, T, const L: usize> DecodeState<F, T, L>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    pub fn new() -> Self {
        Self {
            data: PartialData::<T, L>::new(),
            cursor: DecodeCursor::Init,
        }
    }
}

impl<F, T, const L: usize> DecodeCursor<F, T, L>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData<T, L>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        Self::fields_from(format, reader, 0, data, cx)
    }

    fn fields_from<R>(format: &F, reader: &mut R, idx: usize, data: &mut PartialData<T, L>, cx: &mut Context<'_>) -> Result<Self, <F as backend::Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        for i in idx..L {
            match <T as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                Ok(status) => match status {
                    backend::DecodeStatus::Ready(d) => { data[i] = Some(d); continue },
                    backend::DecodeStatus::Pending(p) => return Ok(Self::Cur(i, p)),
                },
                Err(e) => return Err(e),
            }
        }

        Ok(Self::Fini)
    }
}

pub struct Decode<F, T, const L: usize>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    state: Option<DecodeState<F, T, L>>,
}

impl<F, T, const L: usize> backend::Decode for Decode<F, T, L>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    type Data = Data<T, L>;
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
                    DecodeCursor::Cur(idx, dec) => {
                        futures::ready!(dec.poll_decode(format, reader, cx))
                        .and_then(|d| {
                            data[*idx] = Some(d);
                            DecodeCursor::fields_from(format, reader, *idx + 1, data, cx)
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
        // array items have been created, and this next statement consumes the outer
        // state in order to produce the returned array
        Poll::Ready(Ok(self.state.take().unwrap().data.into_data()))
    }
}

impl<T, const L: usize> backend::Decodable for Data<T, L>
where
    T: backend::Decodable,
{
    type Decoder<F: backend::FormatDecode> = Decode<F, T, L>;
}

impl<T, const L: usize> backend::AsyncDeserialize for Data<T, L>
where
    T: backend::AsyncDeserialize,
{
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