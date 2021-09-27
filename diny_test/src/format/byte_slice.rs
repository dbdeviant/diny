use core::{pin::Pin, task::{Context, Poll}};
use diny::backend::{self, Encode as _, internal::SequenceLen};
use diny::{buffer, io};
use crate::Formatter as ThisFormat;


type Data = [u8];

pub enum Encoder
{
    Init,
    Len(SequenceLen, <SequenceLen as backend::Encodable>::Encoder<ThisFormat>),
    Cur(buffer::BufferCursor),
    Fini,
}

impl Encoder
{
    fn after_init<W>(format: &ThisFormat, writer: &mut W, data: &Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <ThisFormat as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        let len: SequenceLen = data.len().into();
        match <SequenceLen as backend::Encodable>::Encoder::<ThisFormat>::start_encode(format, writer, &len, cx) {
            backend::StartEncodeStatus::Fini         => Self::after_len(format, writer, *len, data, cx),
            backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self::Len(len, enc)),
            backend::StartEncodeStatus::Error(e)     => backend::StartEncodeStatus::Error(e)
        }
    }

    fn after_len<W>(_format: &ThisFormat, writer: &mut W, len: usize, data: &Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <ThisFormat as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        if len > 0 {
            let mut cur = buffer::BufferCursor::new(data);
            match cur.write_remaining(writer, data, cx) {
                backend::PollEncodeStatus::Fini     => backend::StartEncodeStatus::Fini,
                backend::PollEncodeStatus::Pending  => backend::StartEncodeStatus::Pending(Self::Cur(cur)),
                backend::PollEncodeStatus::Error(e) => backend::StartEncodeStatus::Error(e),
            }
        } else {
            backend::StartEncodeStatus::Fini
        }
    }

    fn poll_cur<W>(_format: &ThisFormat, writer: &mut W, cur: &mut buffer::BufferCursor, data: &Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<ThisFormat as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        cur.write_remaining(writer, data, cx)
    }
}

impl backend::Encode for Encoder
{
    type Data = Data;
    type Format = ThisFormat;

    fn init(_data: &Self::Data) -> Self {
        Self::Init
    }

    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <Self::Format as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        Self::after_init(format, writer, data, cx)
    }

    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<Self::Format as backend::Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        match self {
            Self::Init          => diny::encode_chain!(*self, Self::start_encode(format, writer, data, cx)),
            Self::Len(len, enc) => diny::encode_poll_chain!(*self, enc.poll_encode(format, writer, len, cx), Self::after_len(format, writer, **len, data, cx)),
            Self::Cur(cur)      => diny::encode_poll_fini!(*self, Self::poll_cur(format, writer, cur, data, cx)),
            Self::Fini          => backend::PollEncodeStatus::Error(<Self::Format as backend::Format>::invalid_input_err()),
        }
    }
}

pub struct SerializeAll<'w, W> {
    format: &'w ThisFormat,
    writer: &'w mut W,
    data: &'w Data,
    encoder: Encoder,
}

pub(crate) fn serialize<'w, W>(format: &'w ThisFormat, writer: &'w mut W, data: &'w Data) -> SerializeAll<'w, W>
where
    W: ::diny::io::AsyncWrite + Unpin,
{
    SerializeAll::new(format, writer, data)
}

impl<'w, W> SerializeAll<'w, W> {
    fn new(format: &'w ThisFormat, writer: &'w mut W, data: &'w Data) -> SerializeAll<'w, W>
    where
        W: ::diny::io::AsyncWrite + Unpin,
    {
        SerializeAll {
            format,
            writer,
            data,
            encoder: Encoder::init(data),
        }
    }
}

impl<'w, W> Unpin for SerializeAll<'w, W> {}

impl<'w, W> core::future::Future for SerializeAll<'w, W>
where
    W: ::diny::io::AsyncWrite + Unpin,
{
    type Output = Result<(), <ThisFormat as backend::Format>::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.encoder.poll_encode(this.format, this.writer, this.data, cx).into()
    }
}

