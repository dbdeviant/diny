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
//! Derive [AsyncSerialization] support for the desired data types, or derive just
//! [AsyncSerialize] or [AsyncDeserialize] to limit the support to one-way transfers.
//! 
//! The [Serialize] and [Deserialize] objects returned from the [serializer](serializer::serializer)
//! and [deserializer](deserializer::deserializer) methods implement sinks and streams (respectively)
//! and are the simplest way to serialize and deserialize objects that implement [AsyncSerialization].
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
//! # fn main() {
//! let point = Point { x: 1, y: 2 };
//! 
//! // A format can be any implementation of
//! // diny::backend::{FormatSerialize + FormatDeserialize}.
//! let format = diny_test::format();
//! 
//! // A writer can be any implementation of futures::io::AsyncWrite.
//! // In this case, we're using a Vec for simplicity.
//! let writer = vec!();
//! 
//! // A sink is constructible for any implementor of diny::AsyncSerialize
//! let mut sink = diny::serializer(format, writer).into_sink();
//! block_on(sink.send(point)).unwrap();
//! 
//! // Sinks can be destructed back into the inner serializer
//! let diny::Serializer { format, writer } = sink.try_into_inner().unwrap();
//! 
//! // A reader can be any implementation of futures::io::AsyncBufRead.
//! // In this case, we're using a utility module to convert the bytes
//! // written to the vec into an appropriate reader.
//! let reader = diny::util::AsyncSliceReader::from(&writer[..]);
//! 
//! // A stream is constructible for any implementor of diny::AsyncDeserialize
//! let mut stream = diny::deserializer(format, reader).into_stream();
//! let _: Point = block_on(stream.next()).unwrap();
//! # }
//! ```
//! 
//! The [Serializer] and [Deserializer] objects expose `serialize` and
//! `deserialize` methods respecively, which can be used to interleave
//! different [serializable](AsyncSerialization) objects over
//! the same channel.  This has the added benefit of performing the
//! serialization without the overhead of the ownership transfer
//! imposed by sinks and streams.
//! 
//! ```
//! # #![feature(generic_associated_types)]
//! # extern crate futures;
//! # extern crate diny_core;
//! # extern crate diny_test;
//! #
//! # use futures::executor::block_on;
//! # use diny_test::format;
//! #
//! # #[derive(diny::AsyncSerialization)]
//! # pub struct Point {
//! #     x: i32,
//! #     y: i32,
//! # }
//! #
//! # fn main() {
//! let point = Point { x: 1, y: 2 };
//! let slope: i32 = 3;
//!
//! # let writer = vec!();
//! # let format = format();
//! #
//! let mut serializer = diny::serializer(format, writer);
//! # let diny::Serializer { format: _, writer } = 
//! block_on(async {
//!     serializer.serialize(&point).await?;
//!     serializer.serialize(&slope).await?;
//! #   let _ =
//!     serializer.flush().await
//! #   ?;
//! #   let res: Result<diny::Serializer<diny_test::Formatter, Vec<u8>>, <diny_test::Formatter as diny::backend::Format>::Error> = Ok(serializer);
//! #   res
//! }).unwrap();
//! 
//! # let reader = diny::util::AsyncSliceReader::from(&writer[..]);
//! let mut deserializer = diny::deserializer(format, reader);
//! block_on(async {
//!     deserializer.deserialize::<Point>().await?;
//!     deserializer.deserialize::<i32>().await
//! }).unwrap();
//! # }
//! ```
//!
//! The [AsyncSerialize] and [AsyncDeserialize] traits may also be used manually without
//! building an intermediate [Serializer] or [Deserializer] object.
//! 
//! ```
//! # #![feature(generic_associated_types)]
//! # extern crate futures;
//! # extern crate diny_core;
//! # extern crate diny_test;
//! #
//! # use futures::executor::block_on;
//! use futures::io::AsyncWriteExt;
//! use diny::{AsyncDeserialize, AsyncSerialize};
//! # use diny_test::format;
//!
//! # #[derive(diny::AsyncSerialization)]
//! # pub struct Point {
//! #     x: i32,
//! #     y: i32,
//! # }
//! #
//! # fn main() {
//! let point = Point { x: 1, y: 2 };
//!
//! # let format = format();
//! #
//! #   let mut writer = vec!();
//! let write = point.serialize(&format, &mut writer);
//! block_on(write).unwrap();
//! block_on(writer.flush()).unwrap();
//! 
//! #   let mut reader = diny::util::AsyncSliceReader::from(&writer[..]);
//! let read = Point::deserialize(&format, &mut reader);
//! block_on(read).unwrap();
//! # }
//! ```
//! 
//! Additionally, an object's underlying [Encoder](backend::Encodable::Encoder)
//! and [Decoder](backend::Decodable::Decoder) can be easily incorporated into
//! custom futures.  See the [Serialize] and [Deserialize] implementations
//! for an example of embedding them.
//! 
//! The examples directory contains a demonstration of how to use the `async-compat`
//! crate to interoperate with the popular `tokio` library.
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
