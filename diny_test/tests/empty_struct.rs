#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::*;
use common::empty_struct::*;


#[test]
fn can_serialize_empty_struct() {
    test_serialize_exact::<EmptyStruct, 0>(&EmptyStruct::new());
}
