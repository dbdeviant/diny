use core::{marker::PhantomData, pin::Pin, task::{Context, Poll}};
use crate::backend::{Decode, FormatDeserialize};
use crate::io;


/// Implements a [deserialization future](FormatDeserialize) for any
/// [decoder](Decode).
pub struct DeserializeExact<'r, F, R, Dta, Dec> {
    format: &'r F,
    reader: &'r mut R,
    data: PhantomData<*const Dta>,
    decoder: Dec,
}

impl<'r, F, R, Dta, Dec> DeserializeExact<'r, F, R, Dta, Dec> {
    pub fn new(format: &'r F, reader: &'r mut R, decoder: Dec) -> Self {
        Self {
            format,
            reader,
            data: PhantomData,
            decoder,
        }
    }
}

impl<'r, F, R, Dta, Dec> Unpin for DeserializeExact<'r, F, R, Dta, Dec> {}

impl<'r, F, R, Dta, Dec> core::future::Future for DeserializeExact<'r, F, R, Dta, Dec>
where
    F: FormatDeserialize,
    R: io::AsyncBufRead + Unpin,
    Dec: Decode<Format=F, Data=Dta>,
{
    type Output = Result<Dta, F::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.decoder.poll_decode(this.format, this.reader, cx).into()
    }
}