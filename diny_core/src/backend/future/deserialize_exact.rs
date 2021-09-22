use core::{pin::Pin, task::{Context, Poll}};
use futures::{AsyncRead, AsyncBufRead};
use crate::backend::{self, Decode, FormatDecode, FormatDeserialize};

/// Implements a [deserialization future](FormatDeserialize) for any
/// [decoder](Decode).
pub struct DeserializeExact<'r, F, R, Dta, Dec>
where
    F: FormatDecode,
    Dec: Decode<Format=F, Data=Dta>,
{
    format: &'r F,
    reader: &'r mut R,
    decode: Dec,
}

impl<'r, F, R, Dta, Dec> DeserializeExact<'r, F, R, Dta, Dec>
where
    F: FormatDecode,
    Dec: Decode<Format=F, Data=Dta>,
{
    pub fn new(format: &'r F, reader: &'r mut R, decode: Dec) -> Self {
        Self {
            format,
            reader,
            decode,
        }
    }
}

impl<'r, F, R, Dta, Dec> Unpin for DeserializeExact<'r, F, R, Dta, Dec>
where
    F: FormatDeserialize,
    R: Unpin,
    Dec: Decode<Format=F, Data=Dta>,
{}

impl<'r, F, R, Dta, Dec> core::future::Future for DeserializeExact<'r, F, R, Dta, Dec>
where
    F: FormatDeserialize,
    R: AsyncRead + AsyncBufRead + Unpin,
    Dec: Decode<Format=F, Data=Dta>,
{
    type Output = Result<Dta, F::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        match this.decode.poll_decode(this.format, this.reader, cx) {
            backend::PollDecodeStatus::Fini(d)  => Poll::Ready(Ok(d)),
            backend::PollDecodeStatus::Pending  => Poll::Pending,
            backend::PollDecodeStatus::Error(e) => Poll::Ready(Err(e)),
        }
    }
}