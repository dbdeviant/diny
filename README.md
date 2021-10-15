# diny &emsp; [![Build Status]][ci] [![Latest Version]][crates.io] [![Docs]][docs.rs] [![License Info]][license_file]

[Build Status]: https://img.shields.io/github/workflow/status/dbdeviant/diny/ci/main
[ci]: https://github.com/dbdeviant/diny/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/diny
[crates.io]: https://crates.io/crates/diny
[Docs]: https://img.shields.io/docsrs/diny
[docs.rs]: https://docs.rs/diny
[License Info]: https://img.shields.io/crates/l/diny
[license_file]: ./LICENSE.md

An asynchronous, alloc-free, serialization framework written in 100% safe Rust.

---

_NOTE:  **diny** is currently experimental and not ready for production use. Additionally, it requires bulding with the nightly Rust toolchain until [GAT's](https://github.com/rust-lang/rust/issues/44265) are stabilized._

---
**diny** is a slightly opinionated, asynchronous serialization framework that works very similarly to the popular [Serde](https://serde.rs/) framework.  Its main purpose is to support asynchronous serialization in memory constrained execution environments. Because of that, it makes some slightly different design tradeoffs than Serde.

It's still a very young project and the design is subject to change, but as of now the main differences from Serde are:

- **There is no support for zero-copy deserialization.** If that is desirable then it's more likely that simply serializing and deserializing the data synchronously while transferring the serialized buffer asynchronously is a better fit.

    However, if the need is to slip-stream a binary data structure into an asynchronous protocol (e.g. for control flow) without having to manage additional, temporary buffers in the process, **diny** might be a pretty good fit.
- **diny employs a slightly more constrained data model than Serde.**  It is not a design goal to provide serialization specialization for every unique flavor of types that Rust supports. (e.g. unit vs unit_struct vs unit_variant vs newtype_struct(unit) vs newtype_variant(unit)).

    Importantly, **diny** does not currently support anonymous tuple types other than sequences, and doesn't support newtype tuples of any order greater than one (i.e. newtype_struct, newtype_variant).

    Additionally, **diny** only supports owned data strutures, though this design constraint may be partially relaxed soon (i.e. serialization support only).
- **There is only support for binary serialization** vis-a-vis the AsyncWrite and AsyncBufRead traits. This is may change in the near future, but alternative, complete memory constructs (e.g. String) provide limited benefit for asynchronous protocols (e.g. buffers may be split at byte boundaries that interrupt utf-8 code points)

All that aside, **diny** is still in active design, so decisions on any of the above items could change significantly and without notice.

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

Derive AsyncSerialization support for the desired data types.

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

An example of using it with the [tokio](https://tokio.rs/) library
can be found [here](./diny/examples/tokio.rs).
<br/>

#### License
<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>
<br/>
<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
</sub>