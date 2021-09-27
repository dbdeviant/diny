use core::task::{Context, Poll};
use crate::backend::{Encode, FormatSerialize};
use crate::io;


/// Implements a [serialization future](FormatSerialize) for any
/// [encoder](Encode).
pub struct SerializeAll<'w, F, W, Dta, Enc>
where
    Dta: ?Sized,
{
    format: &'w F,
    writer: &'w mut W,
    data: &'w Dta,
    encoder: Enc,
}

impl<'w, F, W, Dta, Enc> SerializeAll<'w, F, W, Dta, Enc>
where
    Dta: ?Sized,
{
    pub fn new(format: &'w F, writer: &'w mut W, data: &'w Dta, encoder: Enc) -> Self {
        Self{
            format,
            writer,
            data,
            encoder,
        }
    }
}

impl<'w, F, W, Dta, Enc> Unpin for SerializeAll<'w, F, W, Dta, Enc>
where
    Dta: ?Sized,
{}

impl<'w, F, W, Dta, Enc> core::future::Future for SerializeAll<'w, F, W, Dta, Enc>
where
    F: FormatSerialize,
    W: io::AsyncWrite + Unpin,
    Dta: ?Sized,
    Enc: Encode<Format=F, Data=Dta>,
{
    type Output = Result<(), F::Error>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.encoder.poll_encode(this.format, &mut this.writer, this.data, cx).into()
    }
}