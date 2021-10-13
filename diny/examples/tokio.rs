
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