#[macro_use]
mod macros;

#[doc(hidden)] pub mod variant_idx  { usize_wrapper_def!(VariantIdx , serialize_variant_idx , EncodeVariantIdx , SerializeVariantIdx , deserialize_variant_idx , DecodeVariantIdx , DeserializeVariantIdx ); }
#[doc(hidden)] pub mod sequence_len { usize_wrapper_def!(SequenceLen, serialize_sequence_len, EncodeSequenceLen, SerializeSequenceLen, deserialize_sequence_len, DecodeSequenceLen, DeserializeSequenceLen); }

#[doc(inline)] pub use variant_idx::VariantIdx;
#[doc(inline)] pub use sequence_len::SequenceLen;