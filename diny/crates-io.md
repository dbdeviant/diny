# diny

An asynchronous, alloc-free, serialization framework written in 100% safe Rust.

---

_**diny** is currently experimental and not ready for production use. Additionally, it requires bulding with the nightly Rust toolchain until [GAT's](https://github.com/rust-lang/rust/issues/44265) are stabilized._

---

**diny** is a slightly opinionated, asynchronous serialization framework that works very similarly to the popular [Serde](https://serde.rs/) framework.  Its main purpose is to support asynchronous serialization in memory constrained execution environments. Because of that, it makes some slightly different design tradeoffs than Serde.

## Usage

Add a dependency on `diny` and a serializer format in `Cargo.toml`:

```toml
[dependencies]
diny = { version = "0.2", features = ["derive"] }
diny_test = "0.2"
```

Enable [GAT](https://rust-lang.github.io/rfcs/1598-generic_associated_types.html) support

```rust
#![feature(generic_associated_types)]
```

Derive `AsyncSerialization` support for the desired data types.

```rust
use futures::{executor::block_on, SinkExt, StreamExt};

#[derive(diny::AsyncSerialization)]
pub struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 1, y: 2 };

// A format can be any implementation of
// diny::backend::{FormatSerialize + FormatDeserialize}.
let format = diny_test::format();

// A writer can be any implementation of futures::io::AsyncWrite.
// In this case, we're using a Vec for simplicity.
let writer = vec!();

// A sink is constructible for any implementor of diny::AsyncSerialize
let mut sink = diny::serializer(format, writer).into_sink();
block_on(sink.send(point)).unwrap();

// Sinks can be destructed back into the inner serializer
let diny::Serializer { format, writer } = sink.try_into_inner().unwrap();

// A reader can be any implementation of futures::io::AsyncBufRead.
// In this case, we're using a utility module to convert the bytes
// written to the vec into an appropriate reader.
let reader = diny::util::AsyncSliceReader::from(&writer[..]);

// A stream is constructible for any implementor of diny::AsyncDeserialize
let mut stream = diny::deserializer(format, reader).into_stream();
let _: Point = block_on(stream.next()).unwrap();
```