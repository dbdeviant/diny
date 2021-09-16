pub mod empty_enum;
pub mod empty_struct;
pub mod my_enum;
pub mod my_rec;
pub mod method;

#[allow(unused)]
use futures::{executor::block_on, future::join};

#[allow(unused)]
pub const VAR_IDX_LEN: usize = 4;
#[allow(unused)]
pub const SEQ_LEN: usize = 8;

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