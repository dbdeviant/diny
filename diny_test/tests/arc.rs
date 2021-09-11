#![feature(generic_associated_types)]

mod common;

use std::sync::Arc;
use common::*;

#[cfg(feature = "std")]
#[test]
fn can_serialize_refcell() {
    const LEN: usize = 8;
    test_serialize_exact::<Arc<u64>, LEN>(&Arc::new(u64::MIN));
}