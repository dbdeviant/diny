#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::VAR_IDX_LEN;
use common::method::eq::*;


#[test]
fn can_serialize_result_ok() {
    test_serialize_exact::<Result<bool, u64>, {VAR_IDX_LEN + 1}>(Ok(true));
}

#[test]
fn can_serialize_result_err() {
    test_serialize_exact::<Result<bool, u64>, {VAR_IDX_LEN + 8}>(Err(5));
}

#[test]
fn can_serialize_array_of_result() {
    test_serialize::<[Result<bool, u64>; 4]>([Err(5), Ok(true), Ok(false), Err(0)]);
}

#[test]
fn can_serialize_result_of_array() {
    test_serialize::<Result<[bool; 2], ()>>(Ok([true, false]));
}