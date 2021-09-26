#![feature(generic_associated_types)]

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "unsafe_speed"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#![deny(missing_docs)]

//! The core functionality of the `diny` framework.
//!
//! See the main `diny` documentation for project status and general usage
#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[macro_use]
mod macros;

/// Types and traits implemented by backend [formatters](backend::Format)
pub mod backend;
/// Helper modules for implementing buffered serialization primitives
pub mod buffer;
/// Helper modules that may be externally useful
pub mod util;

/// Re-export of io related structures
pub mod io;

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "diny_derive")]
pub use diny_derive::AsyncSerialization;

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "diny_derive")]
pub use diny_derive::AsyncSerialize;

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "diny_derive")]
pub use diny_derive::AsyncDeserialize;

#[doc(inline)]
pub use backend::{
    AsyncDeserialize,
    AsyncSerialize,
    AsyncSerialization,
};