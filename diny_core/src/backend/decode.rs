use core::task::{Context, Poll};
use crate::backend::{Format, FormatDecode};
use crate::io;


/// Contains the resultant state of a [start_decode](Decode) opertation.
///
/// This is very simlar to the [Poll] enum, except that it is flattened
/// and that in the event that the operation is pending, the [decode](Decode)
/// state required to resume the operation is returned.
pub enum StartDecodeStatus<Dta, Dec, Err> {
    /// The operation has successfully completed [decoding](Decode) the data.
    Fini(Dta),

    /// The operation is [pending](Poll) and the provided [decoder](Decode) can be used
    /// to continue reading.
    Pending(Dec),

    /// The operation resulted in an error.
    Error(Err),
}

impl<Dta, Dec, Err> StartDecodeStatus<Dta, Dec, Err> {
    /// Convenience method for functorially mapping either variant to a new status.
    #[inline(always)]
    pub fn bimap<Fdta, Gdec, F, G>(self, f: F, g: G) -> StartDecodeStatus<Fdta, Gdec, Err>
    where
        F: FnOnce(Dta) -> Fdta,
        G: FnOnce(Dec) -> Gdec,
     {
        match self {
            Self::Fini   (dta) => StartDecodeStatus::Fini   (f(dta)),
            Self::Pending(dec) => StartDecodeStatus::Pending(g(dec)),
            Self::Error  (err) => StartDecodeStatus::Error    (err),
        }
    }

    /// Convenience method for functorially mapping either variant to a new status.
    #[inline(always)]
    pub fn and_then<Fdta, Gdec, F, G>(self, f: F, g: G) -> StartDecodeStatus<Fdta, Gdec, Err>
    where
        F: FnOnce(Dta) -> StartDecodeStatus<Fdta, Gdec, Err>,
        G: FnOnce(Dec) -> Gdec,
    {
        match self {
            Self::Fini   (dta) => f(dta),
            Self::Pending(dec) => StartDecodeStatus::Pending(g(dec)),
            Self::Error  (err) => StartDecodeStatus::Error    (err),
        }
    }
}

impl<Dta, Dec, Err> From<Dta> for StartDecodeStatus<Dta, Dec, Err> {
    #[inline(always)]
    fn from(data: Dta) -> Self {
        StartDecodeStatus::Fini(data)
    }
}

impl<Dta, Dec, Err> From<Result<Dta, Err>> for StartDecodeStatus<Dta, Dec, Err> {
    #[inline(always)]
    fn from(result: Result<Dta, Err>) -> Self {
        match result {
            Ok(o)  => StartDecodeStatus::Fini(o),
            Err(e) => StartDecodeStatus::Error(e),
        }
    }
}

/// Contains the resultant state of a [poll_decode](Decode) opertation.
///
/// This is very simlar to the [Poll] enum, except that it is flattened.
pub enum PollDecodeStatus<Dta, Err> {
    /// The operation has successfully completed [decoding](Decode) the data.
    Fini(Dta),

    /// The operation is [pending](Poll) and the same [decoder](Decode) can be
    /// polled later.
    Pending,

    /// The operation resulted in an error.
    Error(Err),
}

impl<Dta, Err> PollDecodeStatus<Dta, Err> {
    /// Maps decoded data to the data structure by applying `f`
    #[inline(always)]
    pub fn map<F, D>(self, f: F) -> PollDecodeStatus<D, Err>
    where
        F: FnOnce(Dta) -> D
    {
        match self {
            PollDecodeStatus::Fini(d)  => PollDecodeStatus::Fini(f(d)),
            PollDecodeStatus::Pending  => PollDecodeStatus::Pending,
            PollDecodeStatus::Error(e) => PollDecodeStatus::Error(e),
        }
    }

    /// Maps any returned error to the error returned by applying `f`
    #[inline(always)]
    pub fn map_err<F, E>(self, f: F) -> PollDecodeStatus<Dta, E>
    where
        F: FnOnce(Err) -> E
    {
        match self {
            PollDecodeStatus::Fini(d)  => PollDecodeStatus::Fini(d),
            PollDecodeStatus::Pending  => PollDecodeStatus::Pending,
            PollDecodeStatus::Error(e) => PollDecodeStatus::Error(f(e)),
        }
    }

    /// Binds the decoded data to the data structure mapped by `f`
    #[inline(always)]
    pub fn and_then<F, D>(self, f: F) -> PollDecodeStatus<D, Err>
    where
        F: FnOnce(Dta) -> PollDecodeStatus<D, Err>
    {
        match self {
            PollDecodeStatus::Fini(d)  => f(d),
            PollDecodeStatus::Pending  => PollDecodeStatus::Pending,
            PollDecodeStatus::Error(e) => PollDecodeStatus::Error(e),
        }
    }

    /// Lifts the [PollDecodeStatus] to a [StartDecodeStatus] by consuming
    /// the passed in `pend` object if necessary
    #[inline(always)]
    pub fn lift<P>(self, pend: P) -> StartDecodeStatus<Dta, P, Err>
    {
        match self {
            PollDecodeStatus::Fini(d)  => StartDecodeStatus::Fini(d),
            PollDecodeStatus::Pending  => StartDecodeStatus::Pending(pend),
            PollDecodeStatus::Error(e) => StartDecodeStatus::Error(e),
        }
    }
}

impl<Dta, Err> From<Dta> for PollDecodeStatus<Dta, Err> {
    #[inline(always)]
    fn from(data: Dta) -> Self {
        PollDecodeStatus::Fini(data)
    }
}

impl<Dta, Err> From<Result<Dta, Err>> for PollDecodeStatus<Dta, Err> {
    #[inline(always)]
    fn from(result: Result<Dta, Err>) -> Self {
        match result {
            Ok(o)  => PollDecodeStatus::Fini(o),
            Err(e) => PollDecodeStatus::Error(e),
        }
    }
}

impl<Dta, Err> From<PollDecodeStatus<Dta, Err>> for Poll<Result<Dta, Err>> {
    #[inline(always)]
    fn from(status: PollDecodeStatus<Dta, Err>) -> Self {
        match status {
            PollDecodeStatus::Fini(d) => Poll::Ready(Ok(d)),
            PollDecodeStatus::Pending => Poll::Pending,
            PollDecodeStatus::Error(e) => Poll::Ready(Err(e)),
        }
    }
}

/// Attempt to decode a data structure from an [asynchronous reader](io::AsyncBufRead)
/// for a particular [format](FormatDecode)
pub trait Decode: Sized {
    /// The concrete [format](FormatDecode) to decode with.
    type Format: FormatDecode;
    
    /// The concrete data structure to decode.
    type Data;

    /// Initialize the internal state of the decoder.
    fn init() -> Self;

    /// Begin decoding bytes for the indicated [format](FormatDecode) from the provided [reader](io::AsyncBufRead).
    ///
    /// This is intended to be overriden whenever an optimized code path exists for the (usual) case where
    /// enough bytes have been buffered into the [reader](io::AsyncBufRead) that the operation will
    /// succeed immediately without [pending](Poll).
    ///
    /// # Implementation
    /// Implementions must ensure that `start_decode` is semantically equivalent to calling
    /// `init` followed by `poll_decode`
    fn start_decode<R>(format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> StartDecodeStatus<Self::Data, Self, <<Self as Decode>::Format as Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    {
        let mut dec = Self::init();
        dec.poll_decode(format, reader, cx)
        .lift(dec)
    }

    /// Continue a [pending](Poll) [Decode] operation.
    fn poll_decode<R>(&mut self, format: &Self::Format, reader: &mut R, cx: &mut Context<'_>) -> PollDecodeStatus<Self::Data, <<Self as Decode>::Format as Format>::Error>
    where
        R: io::AsyncBufRead + Unpin,
    ;
}