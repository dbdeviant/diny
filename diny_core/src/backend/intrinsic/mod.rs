#[doc(hidden)] pub mod empty_struct;
#[doc(hidden)] pub mod option;
#[doc(hidden)] pub mod result;
#[doc(hidden)] pub mod array;

#[cfg(any(feature = "std", feature = "alloc"))]
#[doc(hidden)] pub mod string;