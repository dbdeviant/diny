#![feature(generic_associated_types)]

mod common;

use std::cell::Cell;
use common::*;

#[cfg(feature = "std")]
#[test]
fn can_serialize_refcell() {
    const LEN: usize = 8;
    test_serialize_exact::<Cell<u64>, LEN>(&Cell::new(u64::MIN));
}