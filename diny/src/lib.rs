#![feature(generic_associated_types)]

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "unsafe_speed"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#![deny(missing_docs)]

//! An asynchronous, alloc-free, serialization framework written in 100% safe&#8482; Rust.
//!
//! ### EXPERIMENTAL
//! - `diny` currently requires the nightly Rust toolchain >= 1.56.0 for [GAT](https://github.com/rust-lang/rust/issues/44265) support.
//! - `diny` is still in active design--the API is incomplete and prone to change without notice and without backward compatibility.
//! - no_std support is largely ceremonial at this point as the futures-io _traits_ currently require std.
//! 
//! That being said, it _is_ ready for experimentation and design feedback.
//! # Usage
//!
//! Add a dependency on `diny` and a serializer [format](backend::Format) in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! diny = { version = "0.2", features = ["derive"] }
//! diny_test = "0.2"
//! ```
//!
//! Add GAT support to your project's main file (e.g. main.rs, lib.rs):
//!
//! ```
//! #![feature(generic_associated_types)]
//! ```
//!
//! Profit!
//!
//! ```
//! # #![feature(generic_associated_types)]
//! # extern crate futures;
//! # extern crate diny_core;
//! # extern crate diny_test;
//! #
//! use futures::executor::block_on;
//! use diny::{AsyncSerialize, AsyncDeserialize};
//!
//! #[derive(Debug, PartialEq, AsyncSerialize, AsyncDeserialize)]
//! pub struct Point {
//!     x: i32,
//!     y: i32,
//! }
//! 
//! fn main() {
//!     let point = Point { x: 1, y: 2 };
//! 
//!     // A format can be any implementation of
//!     // diny::backend::{FormatSerialize + FormatDeserialize}.
//!     let format = diny_test::format();
//!
//!     // A writer can be any implementation of futures::io::AsyncWrite.
//!     let mut writer = vec!();
//!     let write = point.serialize(&format, &mut writer);
//!     let _ = block_on(write);
//! 
//!     // A reader can be any implementation of futures::io::AsyncBufRead.
//!     // In this case, we're using a utility module to convert the bytes written
//!     // to the vec into an appropriate reader.
//!     let mut reader = diny::util::AsyncSliceReader::from(&writer[..]);
//!     let read = <Point as AsyncDeserialize>::deserialize(&format, &mut reader);
//!     let deserialized = block_on(read).unwrap();
//!     assert_eq!(point, deserialized);
//! }
//! ```
//! 
//! A streaming interface is also available
//! 
//! ```
//! # #![feature(generic_associated_types)]
//! # extern crate futures;
//! # extern crate diny_core;
//! # extern crate diny_test;
//! #
//! use futures::{executor::block_on, SinkExt, StreamExt};
//!
//! #[derive(diny::AsyncSerialization)]
//! pub struct Point {
//!     x: i32,
//!     y: i32,
//! }
//! 
//! fn main() {
//!     let point = Point { x: 1, y: 2 };
//!
//!     // A sink is constructible for any implementor of diny::AsyncSerialize
//!     let mut sink = diny::serializer(diny_test::format(), vec!()).into_sink();
//!     block_on(sink.send(point));
//! 
//!     // If the sink is finished sending, it can be destructed into the inner Serializer
//!     assert!(sink.is_ready());
//!     let diny::Serializer { format, writer } = sink.try_into_inner().unwrap();
//!     let mut reader = diny::util::AsyncSliceReader::from(&writer[..]);
//! 
//!     // A stream is constructible for any implementor of diny::AsyncDeserialize
//!     let mut stream = diny::deserializer(format, &mut reader).into_stream();
//!     let deserialized: Point = block_on(stream.next()).unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! By default, `diny` builds with (and currently requires) Rust's standard library.  Importantly,
//! the `derive` proc macros are _not_ built by default, and need to be enabled to
//! become available.
//!
//! | Feature        | Description                                                         | Default                       |
//! |----------------|---------------------------------------------------------------------|:-----------------------------:|
//! | `derive`       | Support for deriving [AsyncSerialize] and [AsyncDeserialize] traits | <font size="5">&#9744;</font> |
//! | `unsafe_speed` | Permit using unsafe code to improve performance                     | <font size="5">&#9744;</font> |
//! | `std`          | Support for Rust's standard library                                 | <font size="5">&#9745;</font> |
//! | `alloc`        | Support for memory allocation without full `std` support            | <font size="5">&#9744;</font> |
//! | `test`         | Build the diny_test formatter and re-export it to diny::test        | <font size="5">&#9744;</font> |
//!
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Re-export everything from the core module for convenience
pub use diny_core::*;

// If the test serializer is enabled, pull it in as the 'test' module locally.
#[cfg(feature = "test")]
#[doc(hidden)]
pub mod test {
    pub use diny_test::*;
}
