#![allow(missing_docs)]

#[doc(hidden)] pub mod decode;
#[doc(hidden)] pub mod encode;
#[doc(hidden)] pub mod format;
#[doc(hidden)] pub mod future;
#[doc(hidden)] pub mod collection;
#[doc(hidden)] pub mod intrinsic;
#[doc(hidden)] pub mod primitive;
#[doc(hidden)] pub mod wrapper;
/// Types used to support structural serialization
pub mod internal;

use core::future::Future;
use futures::{AsyncRead, AsyncBufRead, AsyncWrite};

#[doc(inline)]
pub use self::{
    decode::{
        Decode,
        DecodeStatus,
    },
    encode::Encode,
    format::{
        Format,
        FormatDecode,
        FormatDeserialize,
        FormatEncode,
        FormatSerialize
    },
    future::{
        deserialize_exact::DeserializeExact,
        serialize_all::SerializeAll,
    }
};

/// Define the [encoder](Encode) to use for serializing the data type.
pub trait Encodable
{
    /// The concrete [encoder](Encode) to use for serialization
    type Encoder<F>: Encode<Data=Self, Format=F>
    where
        F: FormatEncode,
    ;    
}

/// Define the [decoder](Decode) to use for deserializing the data type.
pub trait Decodable: Sized
{
    /// The concrete [decoder](Decode) to use for deserializaiton
    type Decoder<F>: Decode<Data=Self, Format=F>
    where
        F: FormatDecode,
    ;
}

/// Serialize a data structure asynchronously.
pub trait AsyncSerialize: Encodable
{
    /// The concrete [future](Future) returned by the `serialize` method.
    type Future<'w, F, W>: Future<Output=Result<(), F::Error>> + Unpin
    where
        Self: 'w,
        F: 'w + FormatSerialize,
        W: 'w + AsyncWrite + Unpin,
    ;

    /// Attempt to serialize the type asynchronusly for the indicated [format](Format)
    /// via the provided [asynchronous writer](AsyncWrite).
    fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
    where
        F: FormatSerialize,
        W: AsyncWrite + Unpin,
    ;
}

/// Deserialize a data structure asynchronously.
pub trait AsyncDeserialize: Decodable
{
    /// The concrete [future](Future) returned by the `deserialize` method.
    type Future<'r, F, R>: Future<Output=Result<Self, F::Error>> + Unpin
    where
        F: 'r + FormatDeserialize,
        R: 'r + AsyncRead + AsyncBufRead + Unpin,
    ;

    /// Attempt to deserialize the type asynchronusly for the indicated [format](Format)
    /// via the provided [asynchronous reader](AsyncRead).
    fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
    where
        F: FormatDeserialize,
        R: AsyncRead + AsyncBufRead + Unpin,
    ;
}

/// Marker trait to denote that both [AsyncSerialize] and [AsyncDeserialize]
/// are implemented for the type.
///
/// This is defined by default for any type that implements both
/// [AsyncSerialize] and [AsyncDeserialize].
pub trait AsyncSerialization: AsyncSerialize + AsyncDeserialize {}

impl<T> AsyncSerialization for T where T: AsyncSerialize + AsyncDeserialize {}