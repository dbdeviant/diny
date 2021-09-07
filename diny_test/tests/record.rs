#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

use common::*;
use common::my_rec::*;


#[test]
fn can_serialize_my_rec() {
    let send = MyRec{
        bool_0: false,                      // 1
        u8_1: 2u8.pow(5),                   // 1
        opt_u16_2: Some(2u16.pow(13)),      // VAR_IDX_LEN + 2 = 6
        opt_u32_3: Some(Some(2u32.pow(26))) // 2*VAR_IDX_LEN + 4 = 12
    };                                      // 20

    test_serialize_exact::<MyRec, 20>(&send);
}