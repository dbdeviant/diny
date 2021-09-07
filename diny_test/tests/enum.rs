#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::*;
use common::my_enum::*;


#[test]
fn can_serialize_my_enum_bool_0() {
    let send = MyEnum::Bool0(true);
    test_serialize_exact::<MyEnum, {VAR_IDX_LEN + 1}>(&send);
}

#[test]
fn can_serialize_my_enum_u8_1() {
    let send = MyEnum::U81(2u8.pow(4));
    test_serialize_exact::<MyEnum, {VAR_IDX_LEN + 1}>(&send);
}