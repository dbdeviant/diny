#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::*;
use common::my_enum::*;
use common::my_rec::*;


#[test]
fn can_serialize_none() {
    test_serialize_exact::<Option<u64>, VAR_IDX_LEN>(&None);
}

#[test]
fn can_serialize_option_unit() {
    test_serialize_exact::<Option<()>, VAR_IDX_LEN>(&Some(()));
}

#[test]
fn can_serialize_option_bool() {
    test_serialize_exact::<Option<bool>, {VAR_IDX_LEN + 1}>(&Some(false));
}

#[test]
fn can_serialize_option_u8() {
    test_serialize_exact::<Option<u8>, {VAR_IDX_LEN + 1}>(&Some(0));
}

#[test]
fn can_serialize_option_u16() {
    test_serialize_exact::<Option<u16>, {VAR_IDX_LEN + 2}>(&Some(0));
}

#[test]
fn can_serialize_option_u32() {
    test_serialize_exact::<Option<u32>, {VAR_IDX_LEN + 4}>(&Some(0));
}

#[test]
fn can_serialize_option_u64() {
    test_serialize_exact::<Option<u64>, {VAR_IDX_LEN + 8}>(&Some(0));
}

#[test]
fn can_serialize_option_i8() {
    test_serialize_exact::<Option<i8>, {VAR_IDX_LEN + 1}>(&Some(0));
}

#[test]
fn can_serialize_option_i16() {
    test_serialize_exact::<Option<i16>, {VAR_IDX_LEN + 2}>(&Some(0));
}

#[test]
fn can_serialize_option_i32() {
    test_serialize_exact::<Option<i32>, {VAR_IDX_LEN + 4}>(&Some(0));
}

#[test]
fn can_serialize_option_i64() {
    test_serialize_exact::<Option<i64>, {VAR_IDX_LEN + 8}>(&Some(0));
}

#[test]
fn can_serialize_option_my_enum() {
    let send = Some(MyEnum::Bool0(true));
    test_serialize_exact::<Option<MyEnum>, {2*VAR_IDX_LEN + 1}>(&send);
}

#[test]
fn can_serialize_option_my_rec() {
    let send = Some(MyRec{                  // VAR_IDX_LEN = 4
        bool_0: false,                      // 1
        u8_1: 2u8.pow(5),                   // 1
        opt_u16_2: Some(2u16.pow(13)),      // VAR_IDX_LEN + 2 = 6
        opt_u32_3: Some(Some(2u32.pow(26))) // 2*VAR_IDX_LEN + 4 = 12
    });                                     // 24

    test_serialize_exact::<Option<MyRec>, 24>(&send);
}

#[test]
fn can_serialize_option_array() {
    test_serialize_exact::<Option<[u64; 3]>, {VAR_IDX_LEN+8*3}>(&Some([1,2,3]));
}
