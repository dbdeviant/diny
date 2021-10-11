use core::future::Future;
use crate::backend::{Decode, Encode, internal::{VariantIdx, SequenceLen}};
use crate::io;

/// Base trait common to all formatters.
///
/// Provides the minimal required support for handling the
/// errors encountered during [encoding](Encode) and 
/// [decoding](Decode) operations.
pub trait Format {
    /// The type of errors that can occur during serialization and deserialization
    type Error: From<io::Error>;

    /// The error to return when an internal serialization contract has been violated
    fn invalid_input_err() -> Self::Error;

    /// The error to return when a data contract has been violated
    fn invalid_data_err () -> Self::Error;
}

/// Define the primitive [encoders](Encode) utilized by a [formatter](Format)
pub trait FormatEncode: Format {
    type EncodeUnit: Encode<Data=()  , Format=Self>;
    type EncodeBool: Encode<Data=bool, Format=Self>;

    type EncodeI8  : Encode<Data=i8  , Format=Self>;
    type EncodeI16 : Encode<Data=i16 , Format=Self>;
    type EncodeI32 : Encode<Data=i32 , Format=Self>;
    type EncodeI64 : Encode<Data=i64 , Format=Self>;
    type EncodeI128: Encode<Data=i128, Format=Self>;

    type EncodeU8  : Encode<Data=u8  , Format=Self>;
    type EncodeU16 : Encode<Data=u16 , Format=Self>;
    type EncodeU32 : Encode<Data=u32 , Format=Self>;
    type EncodeU64 : Encode<Data=u64 , Format=Self>;
    type EncodeU128: Encode<Data=u128, Format=Self>;

    type EncodeF32 : Encode<Data=f32 , Format=Self>;
    type EncodeF64 : Encode<Data=f64 , Format=Self>;

    type EncodeByteSlice: Encode<Data=[u8], Format=Self>;

    type EncodeChar  : Encode<Data=char, Format=Self>;
    type EncodeStr   : Encode<Data=str , Format=Self>;
    #[cfg(any(feature = "std", feature = "alloc"))]
    type EncodeString: Encode<Data=String, Format=Self>;

    type EncodeVariantIdx : Encode<Data=VariantIdx , Format=Self>;
    type EncodeSequenceLen: Encode<Data=SequenceLen, Format=Self>;
}

/// Define the primitive serialization methods and the concrete [futures](Future) they return.
pub trait FormatSerialize: FormatEncode {
    type SerializeUnit<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeBool<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    type SerializeI8  <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeI16 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeI32 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeI64 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeI128<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    type SerializeU8  <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeU16 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeU32 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeU64 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeU128<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    type SerializeF32 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeF64 <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    type SerializeByteSlice<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    type SerializeChar  <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeStr   <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    #[cfg(any(feature = "std", feature = "alloc"))]
    type SerializeString<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    type SerializeVariantIdx <'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;
    type SerializeSequenceLen<'w, W>: Future<Output=Result<(), Self::Error>> + Unpin where W: 'w + io::AsyncWrite + Unpin;

    fn serialize_unit<'w, W>(&'w self, writer: &'w mut W, data: &()  ) -> Self::SerializeUnit<'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_bool<'w, W>(&'w self, writer: &'w mut W, data: &bool) -> Self::SerializeBool<'w, W> where W: io::AsyncWrite + Unpin;
 
    fn serialize_i8  <'w, W>(&'w self, writer: &'w mut W, data: &i8  ) -> Self::SerializeI8  <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_i16 <'w, W>(&'w self, writer: &'w mut W, data: &i16 ) -> Self::SerializeI16 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_i32 <'w, W>(&'w self, writer: &'w mut W, data: &i32 ) -> Self::SerializeI32 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_i64 <'w, W>(&'w self, writer: &'w mut W, data: &i64 ) -> Self::SerializeI64 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_i128<'w, W>(&'w self, writer: &'w mut W, data: &i128) -> Self::SerializeI128<'w, W> where W: io::AsyncWrite + Unpin;
 
    fn serialize_u8  <'w, W>(&'w self, writer: &'w mut W, data: &u8  ) -> Self::SerializeU8  <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_u16 <'w, W>(&'w self, writer: &'w mut W, data: &u16 ) -> Self::SerializeU16 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_u32 <'w, W>(&'w self, writer: &'w mut W, data: &u32 ) -> Self::SerializeU32 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_u64 <'w, W>(&'w self, writer: &'w mut W, data: &u64 ) -> Self::SerializeU64 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_u128<'w, W>(&'w self, writer: &'w mut W, data: &u128) -> Self::SerializeU128<'w, W> where W: io::AsyncWrite + Unpin;

    fn serialize_f32 <'w, W>(&'w self, writer: &'w mut W, data: &f32 ) -> Self::SerializeF32 <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_f64 <'w, W>(&'w self, writer: &'w mut W, data: &f64 ) -> Self::SerializeF64 <'w, W> where W: io::AsyncWrite + Unpin;

    fn serialize_byte_slice<'w, W>(&'w self, writer: &'w mut W, data: &'w [u8]) -> Self::SerializeByteSlice<'w, W> where W: io::AsyncWrite + Unpin;

    fn serialize_char  <'w, W>(&'w self, writer: &'w mut W, data: &char     ) -> Self::SerializeChar  <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_str   <'w, W>(&'w self, writer: &'w mut W, data: &'w str   ) -> Self::SerializeStr   <'w, W> where W: io::AsyncWrite + Unpin;
    #[allow(clippy::ptr_arg)]
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn serialize_string<'w, W>(&'w self, writer: &'w mut W, data: &'w String) -> Self::SerializeString<'w, W> where W: io::AsyncWrite + Unpin;

    fn serialize_variant_idx <'w, W>(&'w self, writer: &'w mut W, data: &VariantIdx ) -> Self::SerializeVariantIdx <'w, W> where W: io::AsyncWrite + Unpin;
    fn serialize_sequence_len<'w, W>(&'w self, writer: &'w mut W, data: &SequenceLen) -> Self::SerializeSequenceLen<'w, W> where W: io::AsyncWrite + Unpin;
}

/// Define the primitive [decoders](Decode) utilized by a [formatter](Format)
pub trait FormatDecode: Format {
    type DecodeUnit: Decode<Data=()  , Format=Self>;
    type DecodeBool: Decode<Data=bool, Format=Self>;

    type DecodeI8  : Decode<Data=i8  , Format=Self>;
    type DecodeI16 : Decode<Data=i16 , Format=Self>;
    type DecodeI32 : Decode<Data=i32 , Format=Self>;
    type DecodeI64 : Decode<Data=i64 , Format=Self>;
    type DecodeI128: Decode<Data=i128, Format=Self>;

    type DecodeU8  : Decode<Data=u8  , Format=Self>;
    type DecodeU16 : Decode<Data=u16 , Format=Self>;
    type DecodeU32 : Decode<Data=u32 , Format=Self>;
    type DecodeU64 : Decode<Data=u64 , Format=Self>;
    type DecodeU128: Decode<Data=u128, Format=Self>;

    type DecodeF32 : Decode<Data=f32 , Format=Self>;
    type DecodeF64 : Decode<Data=f64 , Format=Self>;

    type DecodeByteVec: Decode<Data=Vec<u8>, Format=Self>;

    type DecodeChar  : Decode<Data=char  , Format=Self>;
    #[cfg(any(feature = "std", feature = "alloc"))]
    type DecodeString: Decode<Data=String, Format=Self>;

    type DecodeVariantIdx : Decode<Data=VariantIdx , Format=Self>;
    type DecodeSequenceLen: Decode<Data=SequenceLen, Format=Self>;
} 

/// Define the primitive deserialization methods and the concrete [futures](Future) they return.
pub trait FormatDeserialize: FormatDecode {
    type DeserializeUnit<'r, R>: Future<Output=Result<()  , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeBool<'r, R>: Future<Output=Result<bool, Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;

    type DeserializeI8  <'r, R>: Future<Output=Result<i8  , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeI16 <'r, R>: Future<Output=Result<i16 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeI32 <'r, R>: Future<Output=Result<i32 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeI64 <'r, R>: Future<Output=Result<i64 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeI128<'r, R>: Future<Output=Result<i128, Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;

    type DeserializeU8  <'r, R>: Future<Output=Result<u8  , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeU16 <'r, R>: Future<Output=Result<u16 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeU32 <'r, R>: Future<Output=Result<u32 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeU64 <'r, R>: Future<Output=Result<u64 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeU128<'r, R>: Future<Output=Result<u128, Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;

    type DeserializeF32 <'r, R>: Future<Output=Result<f32 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeF64 <'r, R>: Future<Output=Result<f64 , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;

    type DeserializeChar  <'r, R>: Future<Output=Result<char  , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    #[cfg(any(feature = "std", feature = "alloc"))]
    type DeserializeString<'r, R>: Future<Output=Result<String, Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;

    type DeserializeVariantIdx <'r, R>: Future<Output=Result<VariantIdx , Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;
    type DeserializeSequenceLen<'r, R>: Future<Output=Result<SequenceLen, Self::Error>> + Unpin where R: 'r + io::AsyncBufRead + Unpin;

    fn deserialize_unit<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeUnit<'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_bool<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeBool<'r, R> where R: io::AsyncBufRead + Unpin;

    fn deserialize_i8  <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI8  <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_i16 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI16 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_i32 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI32 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_i64 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI64 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_i128<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeI128<'r, R> where R: io::AsyncBufRead + Unpin;

    fn deserialize_u8  <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU8  <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_u16 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU16 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_u32 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU32 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_u64 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU64 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_u128<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeU128<'r, R> where R: io::AsyncBufRead + Unpin;

    fn deserialize_f32 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeF32 <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_f64 <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeF64 <'r, R> where R: io::AsyncBufRead + Unpin;

    fn deserialize_char  <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeChar  <'r, R> where R: io::AsyncBufRead + Unpin;
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn deserialize_string<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeString<'r, R> where R: io::AsyncBufRead + Unpin;

    fn deserialize_variant_idx <'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeVariantIdx <'r, R> where R: io::AsyncBufRead + Unpin;
    fn deserialize_sequence_len<'r, R>(&'r self, reader: &'r mut R) -> Self::DeserializeSequenceLen<'r, R> where R: io::AsyncBufRead + Unpin;
}