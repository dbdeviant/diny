#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
#[macro_use]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
mod test {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::vec::Vec;

    use super::common::SEQ_LEN;
    use super::common::method::eq::*;
    use super::common::my_enum::*;
    use super::common::my_rec::*;
    
    
    #[test]
    fn can_serialize_empty_vec() {
        test_serialize_exact::<Vec<u64>, {SEQ_LEN}>(&Vec::new());
    }
    
    #[test]
    fn can_serialize_vec_single() {
        test_serialize_exact::<Vec<u64>, {SEQ_LEN + 8}>(&vec![5]);
    }
    
    #[test]
    fn can_serialize_vec_multiple() {
        test_serialize_exact::<Vec<u64>, {SEQ_LEN+8*3}>(&vec![1,2,3]);
    }
    
    #[test]
    fn can_serialize_vec_large() {
        test_serialize::<Vec<u64>>(&vec![12345678u64; 30]);
    }
    
    #[test]
    fn can_serialize_vec_of_options() {
        test_serialize::<Vec<Option<u64>>>(&vec![Some(1),None,Some(3)]);
    }
    
    #[test]
    fn can_serialize_vec_of_records() {
        test_serialize::<Vec<MyRec>>(&vec![MyRec::new(); 3]);
    }
    
    #[test]
    fn can_serialize_vec_of_enums() {
        test_serialize::<Vec<MyEnum>>(&vec![MyEnum::Bool0(true), MyEnum::U81(4)]);
    }
}
