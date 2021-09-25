#[macro_use]
mod macros;

pub mod unit;
pub mod bool;
pub mod char;

pub mod i8   { numeric_def!(i8  , 1 ); }
pub mod i16  { numeric_def!(i16 , 2 ); }
pub mod i32  { numeric_def!(i32 , 4 ); }
pub mod i64  { numeric_def!(i64 , 8 ); }
pub mod i128 { numeric_def!(i128, 16); }

pub mod u8   { numeric_def!(u8  , 1 ); }
pub mod u16  { numeric_def!(u16 , 2 ); }
pub mod u32  { numeric_def!(u32 , 4 ); }
pub mod u64  { numeric_def!(u64 , 8 ); }
pub mod u128 { numeric_def!(u128, 16); }

pub mod f32  { numeric_def!(f32 , 4 ); }
pub mod f64  { numeric_def!(f64 , 8 ); }

pub mod variant_idx  { usize_wrapper_def!(diny::backend::internal::VariantIdx , u32, format::u32); }
pub mod sequence_len { usize_wrapper_def!(diny::backend::internal::SequenceLen, u64, format::u64); }