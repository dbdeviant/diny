#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use super::common::SEQ_LEN;
    use super::common::method::ord::*;
    use std::collections::BinaryHeap;

    impl<T: Ord> Iterable for BinaryHeap<T> {
        type Item = T;

        type Iter<'a> 
        where
            Self::Item: 'a
        = std::collections::binary_heap::Iter<'a, T>;

        fn iter(&self) -> Self::Iter<'_> {
            self.iter()
        }
    }
    
    #[test]
    fn can_serialize_empty_binary_heap() {
        test_serialize_exact::<BinaryHeap<u64>, {SEQ_LEN}>(&BinaryHeap::new());
    }
    
    #[test]
    fn can_serialize_binary_heap_single() {
        let mut vd = BinaryHeap::new();
        vd.push(5);
        test_serialize_exact::<BinaryHeap<u64>, {SEQ_LEN + 8}>(&vd);
    }
    
    #[test]
    fn can_serialize_binary_heap_multiple() {
        let mut vd = BinaryHeap::new();
        vd.push(1);
        vd.push(2);
        vd.push(3);
        test_serialize_exact::<BinaryHeap<u64>, {SEQ_LEN+8*3}>(&vd);
    }
}
