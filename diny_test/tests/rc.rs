#![feature(generic_associated_types)]

mod common;

use std::rc::Rc;
use common::*;

#[cfg(feature = "std")]
#[test]
fn can_serialize_rc() {
    const LEN: usize = 8;
    test_serialize_exact::<Rc<u64>, LEN>(&Rc::new(u64::MIN));
}