#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use super::common::SEQ_LEN;
    use super::common::method::eq::*;
    use std::collections::VecDeque;
    
    #[test]
    fn can_serialize_empty_vec_deque() {
        test_serialize_exact::<VecDeque<u64>, {SEQ_LEN}>(VecDeque::new());
    }
    
    #[test]
    fn can_serialize_vec_deque_single() {
        let mut vd = VecDeque::new();
        vd.push_back(5);
        test_serialize_exact::<VecDeque<u64>, {SEQ_LEN + 8}>(vd);
    }
    
    #[test]
    fn can_serialize_vec_deque_multiple() {
        let mut vd = VecDeque::new();
        vd.push_back(1);
        vd.push_back(2);
        vd.push_back(3);
        test_serialize_exact::<VecDeque<u64>, {SEQ_LEN+8*3}>(vd);
    }
}
