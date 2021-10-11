#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::task::Context;
use diny::backend::{self, Decode as _, internal::SequenceLen};
use diny::buffer;
use diny::io;
use crate::Formatter as ThisFormat;


type Data = Vec<u8>;
type ByteEncoder = <ThisFormat as backend::FormatEncode>::EncodeByteSlice;

pub struct Encoder(ByteEncoder);

impl backend::Encode for Encoder
{
    type Data = Data;
    type Format = ThisFormat;

    fn init(data: &Self::Data) -> Self {
        Self(ByteEncoder::init(data))
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <Self::Format as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        ByteEncoder::start_encode(format, writer, data, cx)
        .map_pending(Encoder)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<Self::Format as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.0.poll_encode(format, writer, data, cx)
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
    data: Data,
    cursor: DecodeCursor<F>,
}

impl<F> DecodeState<F>
where
    F: backend::FormatDecode,
{
    pub fn new() -> Self {
        Self {
            data: Data::new(),
            cursor: DecodeCursor::Init,
        }
    }
}

impl<F> DecodeCursor<F>
where
    F: backend::FormatDecode,
{
    fn after_init<R>(format: &F, reader: &mut R, data: &mut Data, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        <SequenceLen as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
        .and_then(
            |len| Self::after_len(format, reader, *len, data, cx),
            Self::Len,
        )
    }

    fn after_len<R>(_format: &F, reader: &mut R, len: usize, data: &mut Data, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
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

pub struct Decoder
{
    state: Option<DecodeState<ThisFormat>>,
}

impl backend::Decode for Decoder
{
    type Data = Data;
    type Format = ThisFormat;

    fn init() -> Self {
        Self { state: Some(DecodeState::new()) }
    }

    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <Self::Format as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        let mut data = Data::new();
        match DecodeCursor::after_init(format, reader, &mut data, cx) {
            backend::StartDecodeStatus::Fini(())        => data.into(),
            backend::StartDecodeStatus::Pending(cursor) => backend::StartDecodeStatus::Pending(Self { state: Some(DecodeState { data, cursor }) }),
            backend::StartDecodeStatus::Error(e)        => backend::StartDecodeStatus::Error(e),
        }
    }

    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <Self::Format as backend::Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        if let Some(state) = &mut self.state {
            match &mut state.cursor {
                DecodeCursor::Init => diny::decode_chain!(state.cursor, DecodeCursor, DecodeCursor::after_init(format, reader, &mut state.data, cx)),
                DecodeCursor::Len(dec) =>
                    diny::decode_poll_chain!(
                        state.cursor,
                        DecodeCursor,
                        dec.poll_decode(format, reader, cx),
                        |len: SequenceLen| {
                            DecodeCursor::after_len(format, reader, *len, &mut state.data, cx)
                        }
                    ),
                DecodeCursor::Cur(cur) =>
                    diny::decode_poll_fini!(
                        state.cursor,
                        DecodeCursor,
                        cur.fill_vec(reader, &mut state.data, cx),
                        |()| ()
                    ),
                DecodeCursor::Fini => return backend::PollDecodeStatus::Error(<Self::Format as backend::Format>::invalid_input_err()),
            }
            // SAFETY:
            // The only way this code gets executed is if the outer state existed and reached
            // the DecodeCursor::Fini state as a result of this call.  That cursor state is
            // only reached once all array items have been created, and this next statement
            // consumes the outer state in order to produce the returned array.
            .and_then(|()| self.state.take().unwrap().data.into())
        } else {
            backend::PollDecodeStatus::Error(<Self::Format as backend::Format>::invalid_input_err())
        }
    }
}