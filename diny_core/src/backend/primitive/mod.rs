#[macro_use]
mod macros;

#[doc(hidden)] pub mod unit { primitive_def!(()  , serialize_unit, EncodeUnit, SerializeUnit, deserialize_unit, DecodeUnit, DeserializeUnit); }
#[doc(hidden)] pub mod bool { primitive_def!(bool, serialize_bool, EncodeBool, SerializeBool, deserialize_bool, DecodeBool, DeserializeBool); }

#[doc(hidden)] pub mod i8   { primitive_def!(i8  , serialize_i8  , EncodeI8  , SerializeI8  , deserialize_i8  , DecodeI8  , DeserializeI8  ); }
#[doc(hidden)] pub mod i16  { primitive_def!(i16 , serialize_i16 , EncodeI16 , SerializeI16 , deserialize_i16 , DecodeI16 , DeserializeI16 ); }
#[doc(hidden)] pub mod i32  { primitive_def!(i32 , serialize_i32 , EncodeI32 , SerializeI32 , deserialize_i32 , DecodeI32 , DeserializeI32 ); }
#[doc(hidden)] pub mod i64  { primitive_def!(i64 , serialize_i64 , EncodeI64 , SerializeI64 , deserialize_i64 , DecodeI64 , DeserializeI64 ); }
#[doc(hidden)] pub mod i128 { primitive_def!(i128, serialize_i128, EncodeI128, SerializeI128, deserialize_i128, DecodeI128, DeserializeI128); }

#[doc(hidden)] pub mod u8   { primitive_def!(u8  , serialize_u8  , EncodeU8  , SerializeU8  , deserialize_u8  , DecodeU8  , DeserializeU8  ); }
#[doc(hidden)] pub mod u16  { primitive_def!(u16 , serialize_u16 , EncodeU16 , SerializeU16 , deserialize_u16 , DecodeU16 , DeserializeU16 ); }
#[doc(hidden)] pub mod u32  { primitive_def!(u32 , serialize_u32 , EncodeU32 , SerializeU32 , deserialize_u32 , DecodeU32 , DeserializeU32 ); }
#[doc(hidden)] pub mod u64  { primitive_def!(u64 , serialize_u64 , EncodeU64 , SerializeU64 , deserialize_u64 , DecodeU64 , DeserializeU64 ); }
#[doc(hidden)] pub mod u128 { primitive_def!(u128, serialize_u128, EncodeU128, SerializeU128, deserialize_u128, DecodeU128, DeserializeU128); }

#[doc(hidden)] pub mod char { primitive_def!(char, serialize_char, EncodeChar, SerializeChar, deserialize_char, DecodeChar, DeserializeChar); }