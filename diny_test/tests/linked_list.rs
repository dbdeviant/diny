#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use super::common::*;
    use std::collections::LinkedList;
    
    #[test]
    fn can_serialize_empty_linked_list() {
        test_serialize_exact::<LinkedList<u64>, {SEQ_LEN}>(&LinkedList::new());
    }
    
    #[test]
    fn can_serialize_linked_list_single() {
        let mut vd = LinkedList::new();
        vd.push_back(5);
        test_serialize_exact::<LinkedList<u64>, {SEQ_LEN + 8}>(&vd);
    }
    
    #[test]
    fn can_serialize_linked_list_multiple() {
        let mut vd = LinkedList::new();
        vd.push_back(1);
        vd.push_back(2);
        vd.push_back(3);
        test_serialize_exact::<LinkedList<u64>, {SEQ_LEN+8*3}>(&vd);
    }
}
