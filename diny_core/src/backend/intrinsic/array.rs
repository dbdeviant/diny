use core::task::Context;
use crate::backend::{self, Encode as _, Decode as _};
use crate::io;


type Data<T, const L: usize> = [T; L];

pub enum Encoder<F, T, const L: usize>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    Init,
    Cur(usize, <T as backend::Encodable>::Encoder<F>),
    Fini,
}

impl<F, T, const L: usize> Encoder<F, T, L>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    fn after_init<W>(format: &F, writer: &mut W, data: &Data<T, L>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        Self::fields_from(format, writer, 0, data, cx)
    }

    fn fields_from<W>(format: &F, writer: &mut W, idx: usize, data: &Data<T, L>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        for (i, d) in data.iter().enumerate().skip(idx) {
            match <T as backend::Encodable>::Encoder::<F>::start_encode(format, writer, d, cx) {
                backend::StartEncodeStatus::Fini         => continue,
                backend::StartEncodeStatus::Pending(enc) => return backend::StartEncodeStatus::Pending(Self::Cur(i, enc)),
                backend::StartEncodeStatus::Error(e)     => return backend::StartEncodeStatus::Error(e),
            }
        }

        backend::StartEncodeStatus::Fini
    }
}

impl<F, T, const L: usize> backend::Encode for Encoder<F, T, L>
where
    F: backend::FormatEncode,
    T: backend::Encodable,
{
    type Data = Data<T, L>;
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
            Self::Cur(idx, enc) => encode_poll_chain!(*self, enc.poll_encode(format, writer, &data[*idx], cx), Self::fields_from(format, writer, *idx + 1, data, cx)),
            Self::Fini          => backend::PollEncodeStatus::Error(F::invalid_input_err()),
        }
    }
}

impl<T, const L: usize> backend::Encodable for Data<T, L>
where
    T: backend::Encodable,
{
    type Encoder<F: backend::FormatEncode> = Encoder<F, T, L>;
}


impl<T, const L: usize> backend::AsyncSerialize for Data<T, L>
where
    T: backend::AsyncSerialize,
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
    fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData<T, L>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        Self::fields_from(format, reader, 0, data, cx)
    }

    fn fields_from<R>(format: &F, reader: &mut R, idx: usize, data: &mut PartialData<T, L>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        for i in idx..L {
            match <T as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                backend::StartDecodeStatus::Fini(d) => { data[i] = Some(d); continue },
                backend::StartDecodeStatus::Pending(dec) => return backend::StartDecodeStatus::Pending(Self::Cur(i, dec)),
                backend::StartDecodeStatus::Error(e) => return backend::StartDecodeStatus::Error(e),
            }
        }

        backend::StartDecodeStatus::Fini(())
    }
}

pub struct Decoder<F, T, const L: usize>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    state: Option<DecodeState<F, T, L>>,
}

impl<F, T, const L: usize> backend::Decode for Decoder<F, T, L>
where
    F: backend::FormatDecode,
    T: backend::Decodable,
{
    type Data = Data<T, L>;
    type Format = F;

    fn init() -> Self {
        Self { state: Some(DecodeState::new()) }
    }

    fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        let mut data = PartialData::new();
        match DecodeCursor::after_init(format, reader, &mut data, cx) {
            backend::StartDecodeStatus::Fini(())        => backend::StartDecodeStatus::Fini(data.into_data()),
            backend::StartDecodeStatus::Pending(cursor) => backend::StartDecodeStatus::Pending(Self { state: Some(DecodeState { data, cursor }) }),
            backend::StartDecodeStatus::Error(e)        => backend::StartDecodeStatus::Error(e),
        }
    }

    fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        if let Some(state) = &mut self.state {
            match &mut state.cursor {
                DecodeCursor::Init => decode_chain!(state.cursor, DecodeCursor, DecodeCursor::after_init(format, reader, &mut state.data, cx)),
                DecodeCursor::Cur(idx, dec) =>
                    decode_poll_chain!(
                        state.cursor,
                        DecodeCursor,
                        dec.poll_decode(format, reader, cx),
                        |d| {
                            state.data[*idx] = Some(d);
                            DecodeCursor::fields_from(format, reader, *idx + 1, &mut state.data, cx)
                        }
                    ),
                DecodeCursor::Fini => return backend::PollDecodeStatus::Error(F::invalid_input_err()),
            }
            // SAFETY:
            // The only way this code gets executed is if the outer state existed and reached
            // the DecodeCursor::Fini state as a result of this call.  That cursor state is
            // only reached once all array items have been created, and this next statement
            // consumes the outer state in order to produce the returned array.
            .map(|()| self.state.take().unwrap().data.into_data())
        } else {
            backend::PollDecodeStatus::Error(F::invalid_input_err())
        }
    }
}

impl<T, const L: usize> backend::Decodable for Data<T, L>
where
    T: backend::Decodable,
{
    type Decoder<F: backend::FormatDecode> = Decoder<F, T, L>;
}

impl<T, const L: usize> backend::AsyncDeserialize for Data<T, L>
where
    T: backend::AsyncDeserialize,
{
    type Future<'r, F, R>
    where
        F: 'r + backend::FormatDeserialize,
        R: 'r + io::AsyncBufRead + Unpin,
    = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: backend::FormatDeserialize,
        R: io::AsyncBufRead + Unpin,
    {
        backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
    }
}