use core::{
    pin::Pin,
    task::{Poll, Context}
};
use crate::{backend::{self, Decode, PollDecodeStatus, StartDecodeStatus}, io};

/// Creates a new (Deserializer) from the specified (format)[backend::FormatDecode]
/// and (reader)[io::AsyncBufRead]
pub fn deserializer<F, R>(format: F, reader: R) -> Deserializer<F, R> {
    Deserializer::new(format, reader)
}

/// A wrapper type around a specific (format)[backend::FormatDecode] and (reader)[io::AsyncBufRead]
pub struct Deserializer<F, R> {
    /// The (format)[backend::FormatDecode] used for [decoding](backend::Decode)
    pub format: F,
    /// The (reader)[io::AsyncBufRead] to read (deserialized)[backend::AsyncDeserialize] bytes from
    pub reader: R,
}


impl<F, R> Deserializer<F, R> {
    /// Instantiates a new (Deserializer) from the (format)[backend::FormatDecode] and (reader)[io::AsyncBufRead]
    pub fn new(format: F, reader: R) -> Deserializer<F, R> {
        Deserializer{
            format,
            reader,
        }
    }

    /// Converts the (Deserializer) into a stream of `D`'s
    pub fn into_stream<D>(self) -> Deserialize<F, R, D>
    where
        F: backend::FormatDecode,
        R: io::AsyncBufRead + Unpin,
        D: backend::Decodable,
    {
        Deserialize::new(self)
    }

    /// Deserializes a single object of type `D`
    pub fn deserialize<D>(&mut self) -> D::Future<'_, F, R>
    where
        F: backend::FormatDeserialize,
        R: io::AsyncBufRead + Unpin,
        D: backend::AsyncDeserialize,
    {
        D::deserialize(&self.format, &mut self.reader)
    }
}

enum State<F, D>
where
    F: backend::FormatDecode,
    D: backend::Decodable,
{
    Ready,
    Pending(D::Decoder<F>),
    Error,
}

/// Implements the [Stream](futures::Stream) trait
pub struct Deserialize<F, R, D>
where
    F: backend::FormatDecode,
    D: backend::Decodable,
{
    deserializer: Deserializer<F, R>,
    state: State<F, D>,
}

impl<F, R, D> Deserialize<F, R, D>
where
    F: backend::FormatDecode,
    D: backend::Decodable,
{
    /// Instantiates a new (Deserializer) for the data type `D` from the given `format` and `reader`
    pub fn new(deserializer: Deserializer<F, R>) -> Self {
        Self {
            deserializer,
            state: State::Ready,
        }
    }

    /// Returns `true` if the stream is ready to receive another item, `false` if not
    pub fn is_ready(&self) -> bool {
        matches!(self.state, State::Ready)
    }

    /// Consumes the stream and attempts to return the underlying decoding `format` and `reader`.
    /// 
    /// If the stream is not in the `Ready` state, the underlying reader will be consumed, and
    /// the underlying `format` will be returned as an error.
    pub fn try_into_inner(self) -> Result<Deserializer<F, R>, F> {
        if let State::Ready = self.state {
            Ok(self.deserializer)
        } else {
            Err(self.deserializer.format)
        }
    }
}

impl<F, R, D> Unpin for Deserialize<F, R, D>
where
    F: backend::FormatDecode,
    D: backend::Decodable,
{}

impl<F, R, D> futures::Stream for Deserialize<F, R, D>
where
    F: backend::FormatDecode,
    R: io::AsyncBufRead + Unpin,
    D: backend::Decodable,
{
    type Item = D;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            deserializer,
            state,
        } = &mut *self;

        match state {
            State::Ready => match D::Decoder::start_decode(&deserializer.format, &mut deserializer.reader, cx) {
                StartDecodeStatus::Fini(d) => Poll::Ready(Some(d)),
                StartDecodeStatus::Pending(dec) => {
                    *state = State::Pending(dec);
                    Poll::Pending
                }
                StartDecodeStatus::Error(_) => {
                    *state = State::Error;
                    Poll::Ready(None)
                }
            }
            State::Pending(p) => match p.poll_decode(&deserializer.format, &mut deserializer.reader, cx) {
                PollDecodeStatus::Fini(d) => {
                    *state = State::Ready;
                    Poll::Ready(Some(d))
                }
                PollDecodeStatus::Pending => Poll::Pending,
                PollDecodeStatus::Error(_) => {
                    *state = State::Error;
                    Poll::Ready(None)
                }
            }
            State::Error => Poll::Ready(None)
        }
    }
}