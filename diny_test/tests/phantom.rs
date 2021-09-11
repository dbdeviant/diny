#![feature(generic_associated_types)]

mod common;

use core::marker::PhantomData;
use common::*;

#[test]
fn can_serialize_phantom_data() {
    const LEN: usize = 0;
    test_serialize_exact::<PhantomData<u64>, LEN>(&PhantomData);
}