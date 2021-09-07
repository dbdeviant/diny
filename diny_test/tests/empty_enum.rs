#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::*;
use common::empty_enum::*;


#[test]
fn can_serialize_empty_enum_unitary() {
    let send = EmptyEnum::Unitary;
    test_serialize_exact::<EmptyEnum, {VAR_IDX_LEN}>(&send);
}

#[test]
fn can_serialize_empty_enum_new_type() {
    let send = EmptyEnum::NewType();
    test_serialize_exact::<EmptyEnum, {VAR_IDX_LEN}>(&send);
}

#[test]
fn can_serialize_empty_enum_anon_type() {
    let send = EmptyEnum::AnonType{};
    test_serialize_exact::<EmptyEnum, {VAR_IDX_LEN}>(&send);
}
