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

    // A reader can be any implementation of futures::io::AsyncBufRead.
    // In this case, we're using a utility module to convert the bytes written
    // to the vec into an appropriate reader.
    let mut reader = diny::util::AsyncSliceReader::from(&writer[..]);
    let read = <Point as AsyncDeserialize>::deserialize(&format, &mut reader);
    let deserialized = block_on(read).unwrap();
    assert_eq!(point, deserialized);
}
```

A streaming interface is also available:

```rust
#![feature(generic_associated_types)]

use futures::{executor::block_on, SinkExt, StreamExt};

#[derive(diny::AsyncSerialization)]
pub struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // A sink is constructible for any implementor of diny::AsyncSerialize
    let mut sink = diny::serializer(diny_test::format(), vec!()).into_sink();
    block_on(sink.send(point));

    // Streams and sinks can be destructed back into the inner Serializer
    let diny::Serializer { format, writer } = sink.try_into_inner().unwrap();
    let mut reader = diny::util::AsyncSliceReader::from(&writer[..]);

    // A stream is constructible for any implementor of diny::AsyncDeserialize
    let mut stream = diny::deserializer(format, &mut reader).into_stream();
    let deserialized: Point = block_on(stream.next()).unwrap();
}
```

And here's an example of using it with tokio:

```rust
#![feature(generic_associated_types)]

use futures::{io, SinkExt, StreamExt};
use tokio::{net, sync::oneshot};
use async_compat::CompatExt;

#[derive(diny::AsyncSerialization, Copy, Clone, PartialEq, Debug)]
pub struct Id(u32);

#[derive(diny::AsyncSerialization, PartialEq, Debug)]
pub struct Ping(Id);

#[derive(diny::AsyncSerialization, PartialEq, Debug)]
pub struct Pong(Id);

const ADDR: &str = "127.0.0.1:8090";

async fn server(ready: oneshot::Sender<()>) -> io::Result<()> {
    let listener = net::TcpListener::bind(ADDR).await?;
    assert!(ready.send(()).is_ok());

    let (mut socket, _) = listener.accept().await?;
    let (rx, tx) = socket.split();

    let mut stream = diny::deserializer(
        diny_test::format(),
        io::BufReader::new(rx.compat()),
    ).into_stream();

    let mut sink = diny::serializer(
        diny_test::format(),
        io::BufWriter::new(tx.compat()),
    ).into_sink();
    
    while let Some(Ping(id)) = stream.next().await {
        sink.send(Pong(id)).await?;
    }
    sink.close().await?;

    Ok(())
}

async fn client(ready: oneshot::Receiver<()>) -> io::Result<()> {
    assert!(ready.await.is_ok());

    let mut socket = net::TcpStream::connect(ADDR).await?;
    let (rx, tx) = socket.split();

    let mut sink = diny::serializer(
        diny_test::format(),
        io::BufWriter::new(tx.compat()),
    ).into_sink();

    let mut stream = diny::deserializer(
        diny_test::format(),
        io::BufReader::new(rx.compat()),
    ).into_stream();

    for i in 0..10 {
        let id = Id(i);
        sink.send(Ping(id)).await?;
        assert_eq!(stream.next().await, Some(Pong(id)));
    }
    sink.close().await?;
    assert_eq!(stream.next().await, None);

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (notify, ready) = oneshot::channel();
    let server = tokio::spawn(server(notify));
    let client = tokio::spawn(client(ready));
    client.await??;
    server.await??;
    
    Ok(())
}
```