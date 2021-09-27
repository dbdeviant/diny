use core::task::{Context, Poll};
use crate::backend::{Format, FormatEncode};
use crate::io;


pub enum StartEncodeStatus<Enc, Err> {
    Fini,
    Pending(Enc),
    Error(Err),
}

impl<Enc, Err> StartEncodeStatus<Enc, Err> {
    #[inline(always)]
    pub fn map_pending<F, E>(self, f: F) -> StartEncodeStatus<E, Err>
    where
        F: FnOnce(Enc) -> E,
    {
        match self {
            Self::Fini         => StartEncodeStatus::Fini,
            Self::Pending(enc) => StartEncodeStatus::Pending(f(enc)),
            Self::Error(e)     => StartEncodeStatus::Error(e),
        }
    }
}

pub enum PollEncodeStatus<Err> {
    Fini,
    Pending,
    Error(Err),
}

impl<Err> PollEncodeStatus<Err> {
    #[inline(always)]
    pub fn lift<Enc>(self, enc: Enc) -> StartEncodeStatus<Enc, Err> {
        match self {
            PollEncodeStatus::Fini       => StartEncodeStatus::Fini,
            PollEncodeStatus::Pending    => StartEncodeStatus::Pending(enc),
            PollEncodeStatus::Error(err) => StartEncodeStatus::Error(err),
        }
    }

    #[inline(always)]
    pub fn map_err<F, E>(self, f: F) -> PollEncodeStatus<E>
    where
        F: FnOnce(Err) -> E,
    {
        match self {
            PollEncodeStatus::Fini => PollEncodeStatus::Fini,
            PollEncodeStatus::Pending => PollEncodeStatus::Pending,
            PollEncodeStatus::Error(err) => PollEncodeStatus::Error(f(err))
        }
    }
}

impl<Err> From<PollEncodeStatus<Err>> for Poll<Result<(), Err>> {
    #[inline(always)]
    fn from(status: PollEncodeStatus<Err>) -> Self {
        match status {
            PollEncodeStatus::Fini     => Poll::Ready(Ok(())),
            PollEncodeStatus::Pending  => Poll::Pending,
            PollEncodeStatus::Error(e) => Poll::Ready(Err(e)),
        }
    }
}

/// Attempt to encode a data structure to an [asynchronous writer](io::AsyncWrite)
/// for a particular [format](FormatEncode).
pub trait Encode: Sized {
    /// The concrete [format](FormatEncode) to encode with.
    type Format: FormatEncode;
    
    /// The concrete data structure to encode.
    type Data: ?Sized;

    /// Initialize the internal state of the encoder.
    fn init(data: &Self::Data) -> Self;

    /// Begin encoding bytes for the indicated [format](FormatEncode) to the provided [writer](io::AsyncWrite).
    ///
    /// This is intended to be overriden whenever an optimized code path exists for the (usual) case where
    /// enough buffer space is available that the operation will succeed immediately without [pending](Poll).
    ///
    /// # Implementation
    /// Implementions must ensure that `start_encode` is semantically equivalent to calling
    /// `init` followed by `poll_encode`
    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> StartEncodeStatus<Self, <<Self as Encode>::Format as Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    {
        let mut enc = Self::init(data);
        enc.poll_encode(format, writer, data, cx)
        .lift(enc)
    }

    /// Continue a [pending](Poll) [encode](FormatEncode) operation.
    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> PollEncodeStatus<<<Self as Encode>::Format as Format>::Error>
    where
        W: io::AsyncWrite + Unpin,
    ;
}





