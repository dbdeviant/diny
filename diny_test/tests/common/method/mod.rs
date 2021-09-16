pub mod eq;
pub mod ord;

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
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
{
    let fmt = &Formatter;

    let mut tx = AsyncSliceWriter::from(buf);
    let write = send.serialize(fmt, &mut tx);

    let write_result = block_on(write);
    write_result.as_ref().expect_err("unexpected success");
}

fn serialize_exact<T, const LEN: usize>(send: &T) -> T
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
{
    let (t, len) = serialize_slice_int(send, &mut [0u8; LEN]);
    assert_eq!(len, LEN);
    t
}

#[allow(dead_code)]
fn serialize_slice<T>(send: &T, buf: &mut [u8]) -> T
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
{
    let (t, _) = serialize_slice_int(send, buf);
    t
}

fn serialize_slice_int<T>(send: &T, buf: &mut [u8]) -> (T, usize)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
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


#[cfg(any(feature = "std", feature = "alloc"))]
fn serialize_vec<T>(send: &T) -> T
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
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
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
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