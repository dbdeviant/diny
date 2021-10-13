#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::method::eq::*;
use common::my_enum::*;
use common::my_rec::*;


#[test]
fn can_serialize_empty_array() {
    test_serialize_exact::<[u64; 0], 0>([]);
}

#[test]
fn can_serialize_array_single() {
    test_serialize_exact::<[u64; 1], 8>([5]);
}

#[test]
fn can_serialize_array_multiple() {
    test_serialize_exact::<[u64; 3], 24>([1,2,3]);
}

#[test]
fn can_serialize_array_large() {
    test_serialize::<[u64; 30]>([12345678u64; 30]);
}

#[test]
fn can_serialize_array_of_options() {
    test_serialize::<[Option<u64>; 3]>([Some(1),None,Some(3)]);
}

#[test]
fn can_serialize_array_of_records() {
    test_serialize::<[MyRec; 3]>([MyRec::new(); 3]);
}

#[test]
fn can_serialize_array_of_enums() {
    test_serialize::<[MyEnum; 2]>([MyEnum::Bool0(true), MyEnum::U81(4)]);
}