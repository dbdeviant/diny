pub mod empty_enum;
pub mod empty_struct;
pub mod my_enum;
pub mod my_rec;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

use diny::util::{AsyncSliceReader, AsyncSliceWriter};

use diny_test::Formatter;
#[allow(unused)]
use futures::{executor::block_on, future::join};

#[allow(unused)]
pub const VAR_IDX_LEN: usize = 4;
#[allow(unused)]
pub const SEQ_LEN: usize = 8;

#[allow(unused)]
pub fn test_serialize<T>(send: &T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + core::fmt::Debug + PartialEq,
{
    #[cfg(any(feature = "std", feature = "alloc"))]
    test_serialize_vec(send);
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    test_serialize_slice(send, &mut [0u8; 1024]);

    #[cfg(feature = "std")]
    test_serialize_pin_hole(send);
}

#[allow(unused)]
pub fn test_serialize_exact<T, const LEN: usize>(send: &T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + core::fmt::Debug + PartialEq,
{
    let len = test_serialize_slice(send, &mut [0u8; LEN]);
    assert_eq!(len, LEN);

    #[cfg(feature = "std")]
    test_serialize_pin_hole(send);
}

fn test_serialize_slice<T>(send: &T, buf: &mut [u8]) -> usize
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + core::fmt::Debug + PartialEq,
{
    let fmt = &Formatter;

    let mut tx = AsyncSliceWriter::from(buf);
    let write = send.serialize(fmt, &mut tx);

    let write_result = block_on(write);
    write_result.as_ref().expect("unable to serialize via slice");
    let bytes_written = tx.bytes_written();

    let mut rx: AsyncSliceReader = tx.as_written().into();
    let read = <T as diny::AsyncDeserialize>::deserialize(fmt, &mut rx);

    let read_result = block_on(read);
    read_result.as_ref().expect("unable to deserialize via slice");
    assert_eq!(rx.bytes_read(), bytes_written);
    assert_eq!(read_result.unwrap(), *send);

    bytes_written
}

#[cfg(any(feature = "std", feature = "alloc"))]
fn test_serialize_vec<T>(send: &T) -> usize
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + core::fmt::Debug + PartialEq,
{
    let fmt = &Formatter;

    let mut tx = Vec::<u8>::new();
    let write = send.serialize(fmt, &mut tx);

    let write_result = block_on(write);
    write_result.as_ref().expect("unable to serialize via vec");
    let bytes_written = tx.len();

    let mut rx: AsyncSliceReader = tx[..].into();
    let read = <T as diny::AsyncDeserialize>::deserialize(fmt, &mut rx);

    let read_result = block_on(read);
    read_result.as_ref().expect("unable to deserialize via vec");
    assert_eq!(rx.bytes_read(), bytes_written);
    assert_eq!(read_result.unwrap(), *send);

    bytes_written
}

#[cfg(feature = "std")]
fn test_serialize_pin_hole<T>(send: &T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + core::fmt::Debug + PartialEq,
{
    let fmt = &Formatter;
    let (mut tx, mut rx) = pin_hole::channel();
    
    let write = send.serialize(fmt, &mut tx);
    let read = <T as diny::AsyncDeserialize>::deserialize(fmt, &mut rx);

    let transfer = block_on(join(write, read));
    transfer.0.as_ref().expect("unable to serialize via pinhole");
    transfer.1.as_ref().expect("unable to deserialize via pinhole");
    assert_eq!(transfer.1.unwrap(), *send);
}

mod pin_hole {
    use core::{pin::Pin, task::{Context, Poll}};
    use futures::{AsyncRead, AsyncWrite, io::BufReader, Stream};
    use futures::channel::mpsc;

    /// Returns a writer, reader pair that will only transmit 1 byte at a time
    pub fn channel() -> (Writer, BufReader<Reader>) {
        let (tx, rx) = mpsc::channel::<u8>(1);
        let writer = Writer(tx);
        let reader = Reader(rx);
        let buf_reader = BufReader::with_capacity(1, reader);
    
        (writer, buf_reader)
    }

    pub struct Writer(mpsc::Sender<u8>);

    impl AsyncWrite for Writer {
        fn poll_write(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<futures::io::Result<usize>>
        {
            assert!(!buf.is_empty());
    
            Poll::Ready(
                futures::ready!(self.0.poll_ready(cx))
                .and_then(|()| self.0.start_send(buf[0]))
                .map_or_else(
                    |_| Err(futures::io::ErrorKind::UnexpectedEof.into()),
                    |_| Ok(1),
                ),
            )
        }
    
        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<futures::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    
        fn poll_close(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<futures::io::Result<()>> {
            self.0.close_channel();
            Poll::Ready(Ok(()))
        }
    }
    
    pub struct Reader(mpsc::Receiver<u8>);
    
    impl AsyncRead for Reader {
        fn poll_read(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &mut [u8],
            ) -> Poll<futures::io::Result<usize>>
        {
            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            match futures::ready!(Pin::new(&mut self.0).poll_next(cx)) {
                None => Poll::Ready(Err(futures::io::ErrorKind::UnexpectedEof.into())),
                Some(b) => {
                    buf[0] = b;
                    Poll::Ready(Ok(1))
                }
            }
        }
    }
}