use core::task::{Context, Poll};
use futures::AsyncWrite;
use crate::backend::{Encode, FormatEncode, FormatSerialize};

/// Implements a [serialization future](FormatSerialize) for any
/// [encoder](Encode).
pub struct SerializeAll<'w, F, W, Dta, Enc>
where
    F: FormatEncode,
    Enc: Encode<Format=F, Data=Dta>,
{
    format: &'w F,
    writer: &'w mut W,
    data: &'w Dta,
    encode: Enc,
}

impl<'w, F, W, Dta, Enc> SerializeAll<'w, F, W, Dta, Enc>
where
    F: FormatSerialize<'w>,
    Enc: Encode<Format=F, Data=Dta>,
{
    pub fn new(format: &'w F, writer: &'w mut W, data: &'w Dta, encode: Enc) -> Self {
        Self{
            format,
            writer,
            data,
            encode,
        }
    }
}

impl<'w, F, W, Dta, Enc> Unpin for SerializeAll<'w, F, W, Dta, Enc>
where
    F: FormatSerialize<'w>,
    W: Unpin,
    Enc: Encode<Format=F, Data=Dta>,
{}

impl<'w, F, W, Dta, Enc> core::future::Future for SerializeAll<'w, F, W, Dta, Enc>
where
    F: FormatSerialize<'w>,
    W: AsyncWrite + Unpin,
    Enc: Encode<Format=F, Data=Dta>,
{
    type Output = Result<(), F::Error>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.encode.poll_encode(this.format, &mut this.writer, this.data, cx)
    }
}