use core::task::Context;
use futures::AsyncWrite;
use crate::backend::{Format, FormatEncode};


pub enum StartEncodeStatus<Enc, Err> {
    Fini,
    Pending(Enc),
    Error(Err),
}

impl<Enc, Err> StartEncodeStatus<Enc, Err> {
    pub fn map_pending<F, Enc2>(self, f: F) -> StartEncodeStatus<Enc2, Err>
    where
        F: FnOnce(Enc) -> Enc2,
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

/// Attempt to encode a data structure to an [asynchronous writer](AsyncWrite)
/// for a particular [format](FormatEncode).
pub trait Encode: Sized {
    /// The concrete [format](FormatEncode) to encode with.
    type Format: FormatEncode;
    
    /// The concrete data structure to encode.
    type Data;

    /// Initialize the internal state of the encoder.
    fn init(data: &Self::Data) -> Self;

    /// Begin encoding bytes for the indicated [format](FormatEncode) to the provided [writer](AsyncWrite).
    ///
    /// This is intended to be overriden whenever an optimized code path exists for the (usual) case where
    /// enough buffer space is available that the operation will succeed immediately without [pending](Poll).
    ///
    /// # Implementation
    /// Implementions must ensure that `start_encode` is semantically equivalent to calling
    /// `init` followed by `poll_encode`
    fn start_encode<W>(format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> StartEncodeStatus<Self, <<Self as Encode>::Format as Format>::Error>
    where
        W: AsyncWrite + Unpin,
    ;

    /// Continue a [pending](Poll) [encode](FormatEncode) operation.
    fn poll_encode<W>(&mut self, format: &Self::Format, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> PollEncodeStatus<<<Self as Encode>::Format as Format>::Error>
    where
        W: AsyncWrite + Unpin,
    ;
}





