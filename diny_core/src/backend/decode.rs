use core::{task::{Context, Poll}};
use futures::{AsyncRead, AsyncBufRead};
use crate::backend::{Format, FormatDecode};

/// Contains the resultant state of a [decode](Decode) opertation.
///
/// This is very simlar to the [Poll] enum, except that in the event
/// that the operation is pending, the [decode](Decode) state required to resume
/// the operation is returned.
pub enum DecodeStatus<Dta, Dec> {
    /// The operation has successfully completed [decoding](Decode) the data.
    Ready  (Dta),
    /// The operation is [pending](Poll) and the provided [decoder](Decode) can be used
    /// to continue reading.
    Pending(Dec),
}

impl<Dta, Dec> DecodeStatus<Dta, Dec> {
    /// Convenience method for functorially mapping either variant to a new status.
    pub fn bimap<Fdta, Gdec, F: FnOnce(Dta) -> Fdta, G: FnOnce(Dec) -> Gdec>(self, f: F, g: G) -> DecodeStatus<Fdta, Gdec> {
        match self {
            Self::Ready  (dta) => DecodeStatus::Ready  (f(dta)),
            Self::Pending(dec) => DecodeStatus::Pending(g(dec)),
        }
    }
}

/// Attempt to decode a data structure from an [asynchronous reader](AsyncRead)
/// for a particular [format](FormatDecode)
pub trait Decode: Sized {
    /// The concrete [format](FormatDecode) to decode with.
    type Format: FormatDecode;
    /// The concrete data structure to decode.
    type Data;

    /// Initialize the internal state of the decoder.
    fn init() -> Self;

    /// Begin decoding bytes for the indicated [format](FormatDecode) from the provided [reader](AsyncRead).
    ///
    /// This is intended to be overriden whenever an optimized code path exists for the (usual) case where
    /// enough bytes have been buffered into the [reader](AsyncBufRead) that the operation will
    /// succeed immediately without [pending](Poll).
    ///
    /// # Implementation
    /// Implementions must ensure that `start_decode` is semantically equivalent to calling
    /// `init` followed by `poll_decode`
    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> Result<DecodeStatus<Self::Data, Self>, <<Self as Decode>::Format as Format>::Error>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        let mut decode = Self::init();
        match decode.poll_decode(format, reader, cx) {
            Poll::Ready(d) => d.map(DecodeStatus::Ready),
            Poll::Pending => Ok(DecodeStatus::Pending(decode)),
        }
    }

    /// Continue a [pending](Poll) [decode](FormatDecode) operation.
    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> Poll<Result<Self::Data, <<Self as Decode>::Format as Format>::Error>>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    ;
}