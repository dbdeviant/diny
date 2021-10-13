pub mod eq;
pub mod ord;
pub mod set;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

use diny::util::{AsyncSliceReader, AsyncSliceWriter};
use diny_test::Formatter;
use crate::common::*;

fn cmp_eq<T: PartialEq + core::fmt::Debug>(t1: &T, t2: &T) -> bool {
    assert_eq!(t1, t2);
    t1 == t2
}

fn serialize_slice_err<T>(send: &T, buf: &mut [u8])
where
    T: diny::AsyncSerialization,
{
    let fmt = &Formatter;

    let mut tx = AsyncSliceWriter::from(buf);
    let write = send.serialize(fmt, &mut tx);

    let write_result = block_on(write);
    write_result.as_ref().expect_err("unexpected success");
}

fn stream_exact<T, const LEN: usize>(send: T) -> T
where
    T: diny::AsyncSerialization,
{
    let (t, len) = stream_slice_int(send, &mut [0u8; LEN]);
    assert_eq!(len, LEN);
    t
}

fn serialize_exact_ref<T, const LEN: usize>(send: &T) -> T
where
    T: diny::AsyncSerialization,
{
    let (t, len) = serialize_slice_int(send, &mut [0u8; LEN]);
    assert_eq!(len, LEN);
    t
}

#[allow(dead_code)]
fn serialize_slice<T>(send: &T, buf: &mut [u8]) -> T
where
    T: diny::AsyncSerialization,
{
    let (t, _) = serialize_slice_int(send, buf);
    t
}

fn serialize_slice_int<T>(send: &T, buf: &mut [u8]) -> (T, usize)
where
    T: diny::AsyncSerialization,
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

    (read_result.unwrap(), bytes_written)
}

#[allow(dead_code)]
fn stream_slice<T>(send: T, buf: &mut [u8]) -> T
where
    T: diny::AsyncSerialization,
{
    let (t, _) = stream_slice_int(send, buf);
    t
}

fn stream_slice_int<T>(send: T, buf: &mut [u8]) -> (T, usize)
where
    T: diny::AsyncSerialization,
{
    use futures::SinkExt;
    use futures::StreamExt;

    let fmt = Formatter;

    let tx = AsyncSliceWriter::from(buf);
    let write = async move {
        let mut sink = diny::serializer(fmt, tx).into_sink();
        let ret = sink.send(send).await;
        assert!(sink.is_ready());
        match ret {
            Ok(()) => sink.try_into_inner(),
            Err(_) => sink.try_into_inner(),
        }
    };

    let write_result = block_on(write);
    write_result.as_ref().expect("unable to serialize via sink");
    let diny::Serializer { format, writer } = write_result.unwrap();
    let bytes_written = writer.bytes_written();

    let rx: AsyncSliceReader = writer.as_written().into();
    let read = async move {
        let mut stream = diny::deserializer(format, rx).into_stream();
        let t = stream.next().await;
        assert!(stream.is_ready());
        stream.try_into_inner().map(|s| (s, t))
    };

    let read_result = block_on(read);
    read_result.as_ref().expect("unable to deserialize via stream");
    let ( diny::Deserializer { format: _, reader }, t ) = read_result.unwrap();
    t.as_ref().expect("stream returned None");
    assert_eq!(reader.bytes_read(), bytes_written);

    (t.unwrap(), bytes_written)
}


#[cfg(any(feature = "std", feature = "alloc"))]
fn stream<T>(send: T) -> T
where
    T: diny::AsyncSerialization,
{
    use futures::SinkExt;
    use futures::StreamExt;

    let write = async move {
        let mut sink = diny::serializer(Formatter, Vec::<u8>::new()).into_sink();
        let ret = sink.send(send).await;
        assert!(sink.is_ready());
        match ret {
            Ok(()) => sink.try_into_inner(),
            Err(_) => sink.try_into_inner(),
        }
    };

    let write_result = block_on(write);
    write_result.as_ref().expect("unable to serialize via sink");
    let diny::Serializer { format, writer } = write_result.unwrap();

    let rx: AsyncSliceReader = writer[..].into();
    let read = async move {
        let mut stream = diny::deserializer(format, rx).into_stream::<T>();
        let t = stream.next().await;
        assert!(stream.is_ready());
        t
    };

    let read_result = block_on(read);
    read_result.as_ref().expect("unable to deserialize via stream");
    read_result.unwrap()
}

#[cfg(any(feature = "std", feature = "alloc"))]
fn serialize_vec<T>(send: &T) -> T
where
    T: diny::AsyncSerialization,
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

    read_result.unwrap()
}

#[cfg(feature = "std")]
fn serialize_pin_hole<T>(send: &T) -> T
where
    T: diny::AsyncSerialization,
{
    let fmt = &Formatter;
    let (mut tx, mut rx) = pin_hole::channel();
    
    let write = send.serialize(fmt, &mut tx);
    let read = <T as diny::AsyncDeserialize>::deserialize(fmt, &mut rx);

    let transfer = block_on(join(write, read));
    transfer.0.as_ref().expect("unable to serialize via pinhole");
    transfer.1.as_ref().expect("unable to deserialize via pinhole");
    transfer.1.unwrap()
}