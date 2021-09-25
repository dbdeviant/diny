#![cfg_attr(not(feature = "std"), no_std)]

#![feature(generic_associated_types)]

mod common;

use common::method::eq::*;

#[test]
fn can_serialize_unit() {
    test_serialize_exact::<(), 0>(&());
}

#[test]
fn can_serialize_bool() {
    const LEN: usize = 1;
    test_serialize_exact::<bool, LEN>(&true);
    test_serialize_exact::<bool, LEN>(&false);

}

#[test]
fn can_serialize_u8() {
    const LEN: usize = 1;
    test_serialize_exact::<u8 , LEN>(& u8::MIN   );
    test_serialize_exact::<u8 , LEN>(&(u8::MIN+1));
    test_serialize_exact::<u8 , LEN>(&(u8::MAX-1));
    test_serialize_exact::<u8 , LEN>(& u8::MAX   );
}

#[test]
fn can_serialize_u16() {
    const LEN: usize = 2;
    test_serialize_exact::<u16, LEN>(& u16::MIN   );
    test_serialize_exact::<u16, LEN>(&(u16::MIN+1));
    test_serialize_exact::<u16, LEN>(&(u16::MAX-1));
    test_serialize_exact::<u16, LEN>(& u16::MAX   );
}

#[test]
fn can_serialize_u32() {
    const LEN: usize = 4;
    test_serialize_exact::<u32, LEN>(& u32::MIN   );
    test_serialize_exact::<u32, LEN>(&(u32::MIN+1));
    test_serialize_exact::<u32, LEN>(&(u32::MAX-1));
    test_serialize_exact::<u32, LEN>(& u32::MAX   );
}

#[test]
fn can_serialize_u64() {
    const LEN: usize = 8;
    test_serialize_exact::<u64, LEN>(& u64::MIN   );
    test_serialize_exact::<u64, LEN>(&(u64::MIN+1));
    test_serialize_exact::<u64, LEN>(&(u64::MAX-1));
    test_serialize_exact::<u64, LEN>(& u64::MAX   );
}

#[test]
fn can_serialize_u128() {
    const LEN: usize = 16;
    test_serialize_exact::<u128, LEN>(& u128::MIN   );
    test_serialize_exact::<u128, LEN>(&(u128::MIN+1));
    test_serialize_exact::<u128, LEN>(&(u128::MAX-1));
    test_serialize_exact::<u128, LEN>(& u128::MAX   );
}

#[test]
fn can_serialize_i8() {
    const LEN: usize = 1;
    test_serialize_exact::<i8 , LEN>(& i8::MIN   );
    test_serialize_exact::<i8 , LEN>(&(i8::MIN+1));
    test_serialize_exact::<i8 , LEN>(&(i8::MAX-1));
    test_serialize_exact::<i8 , LEN>(& i8::MAX   );
    test_serialize_exact::<i8 , LEN>(&-1);
    test_serialize_exact::<i8 , LEN>(&0);
    test_serialize_exact::<i8 , LEN>(&1);
}

#[test]
fn can_serialize_i16() {
    const LEN: usize = 2;
    test_serialize_exact::<i16, LEN>(& i16::MIN   );
    test_serialize_exact::<i16, LEN>(&(i16::MIN+1));
    test_serialize_exact::<i16, LEN>(&(i16::MAX-1));
    test_serialize_exact::<i16, LEN>(& i16::MAX   );
    test_serialize_exact::<i16, LEN>(&-1);
    test_serialize_exact::<i16, LEN>(&0);
    test_serialize_exact::<i16, LEN>(&1);
}

#[test]
fn can_serialize_i32() {
    const LEN: usize = 4;
    test_serialize_exact::<i32, LEN>(& i32::MIN   );
    test_serialize_exact::<i32, LEN>(&(i32::MIN+1));
    test_serialize_exact::<i32, LEN>(&(i32::MAX-1));
    test_serialize_exact::<i32, LEN>(& i32::MAX   );
    test_serialize_exact::<i32, LEN>(&-1);
    test_serialize_exact::<i32, LEN>(&0);
    test_serialize_exact::<i32, LEN>(&1);
}

#[test]
fn can_serialize_i64() {
    const LEN: usize = 8;
    test_serialize_exact::<i64, LEN>(& i64::MIN   );
    test_serialize_exact::<i64, LEN>(&(i64::MIN+1));
    test_serialize_exact::<i64, LEN>(&(i64::MAX-1));
    test_serialize_exact::<i64, LEN>(& i64::MAX   );
    test_serialize_exact::<i64, LEN>(&-1);
    test_serialize_exact::<i64, LEN>(&0);
    test_serialize_exact::<i64, LEN>(&1);
}

#[test]
fn can_serialize_i128() {
    const LEN: usize = 16;
    test_serialize_exact::<i128, LEN>(& i128::MIN   );
    test_serialize_exact::<i128, LEN>(&(i128::MIN+1));
    test_serialize_exact::<i128, LEN>(&(i128::MAX-1));
    test_serialize_exact::<i128, LEN>(& i128::MAX   );
}
