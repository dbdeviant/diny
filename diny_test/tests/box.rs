#![feature(generic_associated_types)]

mod common;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use common::method::eq::*;

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn can_serialize_box() {
    const LEN: usize = 8;
    test_serialize_exact::<Box<u64>, LEN>(&Box::new(u64::MIN));
}