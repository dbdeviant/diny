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
diny = { version = "0.1", features = ["derive"] }
diny_test = "0.1"
```

Turn on the feature for GAT's and derive AsyncSerialize / AsyncDeserialize
on the desired data types.

```rust
#![feature(generic_associated_types)]

use futures::executor::block_on;
use diny::{AsyncSerialize, AsyncDeserialize};

#[derive(Debug, PartialEq, AsyncSerialize, AsyncDeserialize)]
pub struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // A format can be any implementation of
    // diny::backend::{FormatSerialize + FormatDeserialize}.
    let format = diny_test::format();

    // A writer can be any implementation of futures::io::AsyncWrite.
    let mut writer = vec!();
    let write = point.serialize(&format, &mut writer);
    let _ = block_on(write);

    // A reader can be any implementation of futures::io::{AsyncRead + AsyncBufRead}.
    // In this case, we're using a utility module to convert the bytes written
    // to the vec into an appropriate reader.
    let mut reader = diny::util::AsyncSliceReader::from(&writer[..]);
    let read = <Point as AsyncDeserialize>::deserialize(&format, &mut reader);
    let deserialized = block_on(read).unwrap();
    assert_eq!(point, deserialized);
}
```