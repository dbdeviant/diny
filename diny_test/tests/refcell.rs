#![feature(generic_associated_types)]

mod common;

use std::cell::RefCell;
use common::*;

#[cfg(feature = "std")]
#[test]
fn can_serialize_ref_cell() {
    const LEN: usize = 8;
    test_serialize_exact::<RefCell<u64>, LEN>(&RefCell::new(u64::MIN));
}