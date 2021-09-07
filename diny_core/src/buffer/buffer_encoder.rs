use core::{marker::PhantomData, pin::Pin, task::{Context, Poll}};
use futures::AsyncWrite;
use crate::backend::{FormatEncode, FormatSerialize};
use crate::buffer::BufferEncode;

/// A convenience structure that can serialize any implemenation of [BufferEncode].
pub struct BufferEncoder<'w, F, W, Dta, Enc>
where
    F: FormatEncode,
    Enc: BufferEncode<Format=F, Data=Dta>,
{
    format: &'w F,
    writer: &'w mut W,
    data: PhantomData::<&'w Dta>,
    encode: Enc,
}

impl<'w, F, W, Dta, Enc> BufferEncoder<'w, F, W, Dta, Enc>
where
    F: FormatSerialize<'w>,
    Enc: BufferEncode<Format=F, Data=Dta>,
{
    pub fn new(format: &'w F, writer: &'w mut W, encode: Enc) -> Self {
        Self{
            format,
            writer,
            data: PhantomData::<&'w Dta>,
            encode,
        }
    }
}

impl<'w, F, W, Dta, Enc> Unpin for BufferEncoder<'w, F, W, Dta, Enc>
where
    F: FormatSerialize<'w>,
    W: Unpin,
    Enc: BufferEncode<Format=F, Data=Dta>,
{}

impl<'w, F, W, Dta, Enc> core::future::Future for BufferEncoder<'w, F, W, Dta, Enc>
where
    F: FormatSerialize<'w>,
    W: AsyncWrite + Unpin,
    Enc: BufferEncode<Format=F, Data=Dta>,
{
    type Output = Result<(), F::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.encode.poll_encode_buffer(this.format, &mut this.writer, cx)
    }
}