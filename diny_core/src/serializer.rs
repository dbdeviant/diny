use core::{
    pin::Pin,
    task::{Poll, Context}
};
use crate::{backend::{self, Encode, PollEncodeStatus}, io};

/// Creates a new [Serializer] from the specified [format](backend::FormatEncode)
/// and [writer](io::AsyncWrite)
pub fn serializer<F, W>(format: F, writer: W) -> Serializer<F, W> {
    Serializer::new(format, writer)
}

/// A wrapper type around a specific [format](backend::FormatEncode) and [writer](io::AsyncWrite)
pub struct Serializer<F, W> {
    /// The [format](backend::FormatEncode) used for [encoding](backend::Encode)
    pub format: F,
    /// The [writer](io::AsyncWrite) to write [serialized](backend::AsyncSerialize) bytes to
    pub writer: W,
}

impl<F, W> Serializer<F, W> {
    /// Instantiates a new [Serializer] from the [format](backend::FormatEncode) and [writer](io::AsyncWrite)
    pub fn new(format: F, writer: W) -> Serializer<F, W> {
        Serializer {
            format,
            writer,
        }
    }

    /// Converts the [Serializer] into a sink for `D`'s
    pub fn into_sink<D>(self) -> Serialize<F, W, D>
    where
        F: backend::FormatEncode,
        W: io::AsyncWrite + Unpin,
        D: backend::Encodable,
    {
        Serialize::new(self)
    }

    /// Serializes a single object of type `D`
    pub fn serialize<'w, D>(&'w mut self, data: &'w D) -> D::Future<'w, F, W>
    where
        F: backend::FormatSerialize,
        W: io::AsyncWrite + Unpin,
        D: backend::AsyncSerialize,
    {
        D::serialize(data, &self.format, &mut self.writer)
    }

    /// Flushes the underlying `writer`
    pub fn flush(&mut self) -> impl '_ + futures::Future<Output=Result<(), <F as backend::Format>::Error>>
    where
        F: backend::FormatSerialize,
        W: io::AsyncWrite + Unpin,
    {
        use futures::{io::AsyncWriteExt, TryFutureExt};
        self.writer.flush().map_err(|e| e.into())
    }
}

enum State<F, D>
where
    F: backend::FormatEncode,
    D: backend::Encodable,
{
    Ready,
    Pending(D::Encoder<F>, D),
    Error,
    Closed,
}

/// Implements the [Sink](futures::Sink) trait
pub struct Serialize<F, W, D>
where
    F: backend::FormatEncode,
    D: backend::Encodable,
{
    serializer: Serializer<F, W>,
    state: State<F, D>,
}

impl<F, W, D> Serialize<F, W, D>
where
    F: backend::FormatEncode,
    W: io::AsyncWrite + Unpin,
    D: backend::Encodable,
{
    /// Instantiates a new [Serializer] for the data type `D` from the given `format` and `writer`
    pub fn new(serializer: Serializer<F, W>) -> Self
    where
        W: futures::AsyncWrite + Unpin,
    {
        Self {
            serializer,
            state: State::Ready,
        }
    }

    /// Returns `true` if the sink is ready to send another item, `false` if not
    pub fn is_ready(&self) -> bool {
        matches!(self.state, State::Ready)
    }

    /// Consumes the sink and attempts to return the underlying decoding `format` and `writer`.
    /// 
    /// If the stream is not in the `Ready` state, the underlying writer will be consumed, and
    /// the underlying `format` will be returned as an error.
    pub fn try_into_inner(self) -> Result<Serializer<F, W>, F> {
        if let State::Ready = self.state {
            Ok(self.serializer)
        } else {
            Err(self.serializer.format)
        }
    }
}

impl<F, W, D> Unpin for Serialize<F, W, D>
where
    F: backend::FormatEncode,
    D: backend::Encodable,
{}

impl<F, W, D> futures::Sink<D> for Serialize<F, W, D>
where
    F: backend::FormatEncode,
    W: io::AsyncWrite + Unpin,
    D: backend::Encodable,
{
    type Error = F::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let Self {
            serializer,
            state,
        } = &mut *self;

        match state {
            State::Ready => Poll::Ready(Ok(())),
            State::Pending(enc, data) => match enc.poll_encode(&serializer.format, &mut serializer.writer, data, cx) {
                PollEncodeStatus::Fini => {
                    *state = State::Ready;
                    Poll::Ready(Ok(()))
                }
                PollEncodeStatus::Pending => Poll::Pending,
                PollEncodeStatus::Error(e) => {
                    *state = State::Error;
                    Poll::Ready(Err(e))
                }
            }
            State::Error => Poll::Ready(Err(<F as backend::Format>::invalid_input_err())),
            State::Closed => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: D) -> Result<(), Self::Error> {
        let state = &mut self.state;

        if let State::Ready = state {
            *state = State::Pending(<D::Encoder<F> as Encode>::init(&item), item);
            Ok(())
        } else {
            Err(<F as backend::Format>::invalid_input_err())
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        futures::ready!(Pin::new(&mut *self).poll_ready(cx))?;
        match futures::ready!(Pin::new(&mut self.serializer.writer).poll_flush(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => {
                self.state = State::Error;
                Poll::Ready(Err(e.into()))
            }
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        futures::ready!(Pin::new(&mut *self).poll_ready(cx))?;
        match futures::ready!(Pin::new(&mut self.serializer.writer).poll_close(cx)) {
            Ok(()) => {
                self.state = State::Closed;
                Poll::Ready(Ok(()))
            }
            Err(e) => {
                self.state = State::Error;
                Poll::Ready(Err(e.into()))
            }
        }
    }
}