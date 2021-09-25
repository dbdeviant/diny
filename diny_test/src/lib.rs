#![feature(generic_associated_types)]

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "unsafe_speed"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#![deny(missing_docs)]

//! A formatter for the `diny` framework useful for testing purposes only.
//!
//! See the main `diny` documentation for project status and general usage
#[macro_use]
mod macros;
#[doc(hidden)] pub mod format;

use futures::{AsyncRead, AsyncBufRead, AsyncWrite};

/// Create a test formatter instance
pub fn format() -> Formatter {
    Formatter
}

/// A test format that trivially encodes the primitives as their
/// little endian, in memory byte representations.
pub struct Formatter;

impl diny::backend::Format for Formatter {
    type Error = futures::io::Error;

    fn invalid_input_err() -> Self::Error {
        futures::io::ErrorKind::InvalidInput.into()
    }

    fn invalid_data_err() -> Self::Error {
        futures::io::ErrorKind::InvalidData.into()
    }
}

impl diny::backend::FormatEncode for Formatter {
    type EncodeUnit = format::unit::Encoder;
    type EncodeBool = format::bool::Encoder;

    type EncodeI8   = format::i8  ::Encoder;
    type EncodeI16  = format::i16 ::Encoder;
    type EncodeI32  = format::i32 ::Encoder;
    type EncodeI64  = format::i64 ::Encoder;
    type EncodeI128 = format::i128::Encoder;

    type EncodeU8   = format::u8  ::Encoder;
    type EncodeU16  = format::u16 ::Encoder;
    type EncodeU32  = format::u32 ::Encoder;
    type EncodeU64  = format::u64 ::Encoder;
    type EncodeU128 = format::u128::Encoder;

    type EncodeF32  = format::f32 ::Encoder;
    type EncodeF64  = format::f64 ::Encoder;

    type EncodeChar = format::char::Encoder;

    type EncodeVariantIdx  = format::variant_idx ::Encoder;
    type EncodeSequenceLen = format::sequence_len::Encoder;
}

impl diny::backend::FormatSerialize for Formatter
{
    type SerializeUnit<'w, W> where W: 'w + AsyncWrite + Unpin = format::unit::SerializeAll<'w, W>;
    type SerializeBool<'w, W> where W: 'w + AsyncWrite + Unpin = format::bool::SerializeAll<'w, W>;

    type SerializeI8  <'w, W> where W: 'w + AsyncWrite + Unpin = format::i8  ::SerializeAll<'w, W>;
    type SerializeI16 <'w, W> where W: 'w + AsyncWrite + Unpin = format::i16 ::SerializeAll<'w, W>;
    type SerializeI32 <'w, W> where W: 'w + AsyncWrite + Unpin = format::i32 ::SerializeAll<'w, W>;
    type SerializeI64 <'w, W> where W: 'w + AsyncWrite + Unpin = format::i64 ::SerializeAll<'w, W>;
    type SerializeI128<'w, W> where W: 'w + AsyncWrite + Unpin = format::i128::SerializeAll<'w, W>;

    type SerializeU8  <'w, W> where W: 'w + AsyncWrite + Unpin = format::u8  ::SerializeAll<'w, W>;
    type SerializeU16 <'w, W> where W: 'w + AsyncWrite + Unpin = format::u16 ::SerializeAll<'w, W>;
    type SerializeU32 <'w, W> where W: 'w + AsyncWrite + Unpin = format::u32 ::SerializeAll<'w, W>;
    type SerializeU64 <'w, W> where W: 'w + AsyncWrite + Unpin = format::u64 ::SerializeAll<'w, W>;
    type SerializeU128<'w, W> where W: 'w + AsyncWrite + Unpin = format::u128::SerializeAll<'w, W>;

    type SerializeF32 <'w, W> where W: 'w + AsyncWrite + Unpin = format::f32 ::SerializeAll<'w, W>;
    type SerializeF64 <'w, W> where W: 'w + AsyncWrite + Unpin = format::f64 ::SerializeAll<'w, W>;

    type SerializeChar<'w, W> where W: 'w + AsyncWrite + Unpin = format::char::SerializeAll<'w, W>;

    type SerializeVariantIdx <'w, W> where W: 'w + AsyncWrite + Unpin = format::variant_idx ::SerializeAll<'w, W>;
    type SerializeSequenceLen<'w, W> where W: 'w + AsyncWrite + Unpin = format::sequence_len::SerializeAll<'w, W>;

    fn serialize_unit<'w, W>(&'w self, writer: &'w mut W, data: &()  ) -> Self::SerializeUnit<'w, W> where W: AsyncWrite + Unpin { format::unit::serialize(self, writer, data) } 
    fn serialize_bool<'w, W>(&'w self, writer: &'w mut W, data: &bool) -> Self::SerializeBool<'w, W> where W: AsyncWrite + Unpin { format::bool::serialize(self, writer, data) }
 
    fn serialize_i8  <'w, W>(&'w self, writer: &'w mut W, data: &i8  ) -> Self::SerializeI8  <'w, W> where W: AsyncWrite + Unpin { format::i8  ::serialize(self, writer, data) }
    fn serialize_i16 <'w, W>(&'w self, writer: &'w mut W, data: &i16 ) -> Self::SerializeI16 <'w, W> where W: AsyncWrite + Unpin { format::i16 ::serialize(self, writer, data) }
    fn serialize_i32 <'w, W>(&'w self, writer: &'w mut W, data: &i32 ) -> Self::SerializeI32 <'w, W> where W: AsyncWrite + Unpin { format::i32 ::serialize(self, writer, data) }
    fn serialize_i64 <'w, W>(&'w self, writer: &'w mut W, data: &i64 ) -> Self::SerializeI64 <'w, W> where W: AsyncWrite + Unpin { format::i64 ::serialize(self, writer, data) }
    fn serialize_i128<'w, W>(&'w self, writer: &'w mut W, data: &i128) -> Self::SerializeI128<'w, W> where W: AsyncWrite + Unpin { format::i128::serialize(self, writer, data) }
 
    fn serialize_u8  <'w, W>(&'w self, writer: &'w mut W, data: &u8  ) -> Self::SerializeU8  <'w, W> where W: AsyncWrite + Unpin { format::u8  ::serialize(self, writer, data) }
    fn serialize_u16 <'w, W>(&'w self, writer: &'w mut W, data: &u16 ) -> Self::SerializeU16 <'w, W> where W: AsyncWrite + Unpin { format::u16 ::serialize(self, writer, data) }
    fn serialize_u32 <'w, W>(&'w self, writer: &'w mut W, data: &u32 ) -> Self::SerializeU32 <'w, W> where W: AsyncWrite + Unpin { format::u32 ::serialize(self, writer, data) }
    fn serialize_u64 <'w, W>(&'w self, writer: &'w mut W, data: &u64 ) -> Self::SerializeU64 <'w, W> where W: AsyncWrite + Unpin { format::u64 ::serialize(self, writer, data) }
    fn serialize_u128<'w, W>(&'w self, writer: &'w mut W, data: &u128) -> Self::SerializeU128<'w, W> where W: AsyncWrite + Unpin { format::u128::serialize(self, writer, data) }

    fn serialize_f32 <'w, W>(&'w self, writer: &'w mut W, data: &f32 ) -> Self::SerializeF32 <'w, W> where W: AsyncWrite + Unpin { format::f32 ::serialize(self, writer, data) }
    fn serialize_f64 <'w, W>(&'w self, writer: &'w mut W, data: &f64 ) -> Self::SerializeF64 <'w, W> where W: AsyncWrite + Unpin { format::f64 ::serialize(self, writer, data) }

    fn serialize_char<'w, W>(&'w self, writer: &'w mut W, data: &char) -> Self::SerializeChar<'w, W> where W: AsyncWrite + Unpin { format::char::serialize(self, writer, data) }

    fn serialize_variant_idx <'w, W>(&'w self, writer: &'w mut W, data: &diny::backend::internal::VariantIdx ) -> Self::SerializeVariantIdx <'w, W> where W: AsyncWrite + Unpin { format::variant_idx ::serialize(self, writer, data) }
    fn serialize_sequence_len<'w, W>(&'w self, writer: &'w mut W, data: &diny::backend::internal::SequenceLen) -> Self::SerializeSequenceLen<'w, W> where W: AsyncWrite + Unpin { format::sequence_len::serialize(self, writer, data) }
}

impl diny::backend::FormatDecode for Formatter {
    type DecodeUnit = format::unit::Decoder;
    type DecodeBool = format::bool::Decoder;

    type DecodeI8   = format::i8  ::Decoder;
    type DecodeI16  = format::i16 ::Decoder;
    type DecodeI32  = format::i32 ::Decoder;
    type DecodeI64  = format::i64 ::Decoder;
    type DecodeI128 = format::i128::Decoder;

    type DecodeU8   = format::u8  ::Decoder;
    type DecodeU16  = format::u16 ::Decoder;
    type DecodeU32  = format::u32 ::Decoder;
    type DecodeU64  = format::u64 ::Decoder;
    type DecodeU128 = format::u128::Decoder;

    type DecodeF32  = format::f32 ::Decoder;
    type DecodeF64  = format::f64 ::Decoder;

    type DecodeChar = format::char::Decoder;

    type DecodeVariantIdx  = format::variant_idx ::Decoder;
    type DecodeSequenceLen = format::sequence_len::Decoder;
}

impl diny::backend::FormatDeserialize for Formatter
{
    type DeserializeUnit<'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::unit::DeserializeExact<'r, R>;
    type DeserializeBool<'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::bool::DeserializeExact<'r, R>;

    type DeserializeI8  <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::i8  ::DeserializeExact<'r, R>;
    type DeserializeI16 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::i16 ::DeserializeExact<'r, R>;
    type DeserializeI32 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::i32 ::DeserializeExact<'r, R>;
    type DeserializeI64 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::i64 ::DeserializeExact<'r, R>;
    type DeserializeI128<'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::i128::DeserializeExact<'r, R>;

    type DeserializeU8  <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::u8  ::DeserializeExact<'r, R>;
    type DeserializeU16 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::u16 ::DeserializeExact<'r, R>;
    type DeserializeU32 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::u32 ::DeserializeExact<'r, R>;
    type DeserializeU64 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::u64 ::DeserializeExact<'r, R>;
    type DeserializeU128<'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::u128::DeserializeExact<'r, R>;

    type DeserializeF32 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::f32 ::DeserializeExact<'r, R>;
    type DeserializeF64 <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::f64 ::DeserializeExact<'r, R>;

    type DeserializeChar<'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::char::DeserializeExact<'r, R>;

    type DeserializeVariantIdx <'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::variant_idx ::DeserializeExact<'r, R>;
    type DeserializeSequenceLen<'r, R> where R: 'r + AsyncRead + AsyncBufRead + Unpin = format::sequence_len::DeserializeExact<'r, R>;

    fn deserialize_unit<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeUnit<'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::unit::deserialize(self, reader) }
    fn deserialize_bool<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeBool<'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::bool::deserialize(self, reader) }

    fn deserialize_i8  <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI8  <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::i8  ::deserialize(self, reader) }
    fn deserialize_i16 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI16 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::i16 ::deserialize(self, reader) }
    fn deserialize_i32 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI32 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::i32 ::deserialize(self, reader) }
    fn deserialize_i64 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI64 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::i64 ::deserialize(self, reader) }
    fn deserialize_i128<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI128<'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::i128::deserialize(self, reader) }

    fn deserialize_u8  <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU8  <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::u8  ::deserialize(self, reader) }
    fn deserialize_u16 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU16 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::u16 ::deserialize(self, reader) }
    fn deserialize_u32 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU32 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::u32 ::deserialize(self, reader) }
    fn deserialize_u64 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU64 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::u64 ::deserialize(self, reader) }
    fn deserialize_u128<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU128<'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::u128::deserialize(self, reader) }

    fn deserialize_f32 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeF32 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::f32 ::deserialize(self, reader) }
    fn deserialize_f64 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeF64 <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::f64 ::deserialize(self, reader) }

    fn deserialize_char<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeChar<'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::char::deserialize(self, reader) }

    fn deserialize_variant_idx <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeVariantIdx <'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::variant_idx ::deserialize(self, reader) }
    fn deserialize_sequence_len<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeSequenceLen<'r, R> where R: AsyncRead + AsyncBufRead + Unpin { format::sequence_len::deserialize(self, reader) }
}