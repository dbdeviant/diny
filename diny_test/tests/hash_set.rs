#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use core::hash::{Hash, BuildHasher};
    use std::collections::HashSet;
    use super::common::SEQ_LEN;
    use super::common::method::set::*;

    impl<T, S> IsEquivalentTo for HashSet<T, S>
    where
        T: Eq + Hash,
        S: BuildHasher,
    {
        fn is_equivalent_to(&self, other: &Self) -> bool {
            self.symmetric_difference(other).count() == 0
        }
    }
    
    #[test]
    fn can_serialize_empty_hash_set() {
        test_serialize_exact::<HashSet<u64>, {SEQ_LEN}>(&HashSet::new());
    }
    
    #[test]
    fn can_serialize_hash_set_single() {
        let mut vd = HashSet::new();
        vd.insert(5);
        test_serialize_exact::<HashSet<u64>, {SEQ_LEN + 8}>(&vd);
    }
    
    #[test]
    fn can_serialize_hash_set_multiple() {
        let mut vd = HashSet::new();
        vd.insert(1);
        vd.insert(2);
        vd.insert(3);
        test_serialize_exact::<HashSet<u64>, {SEQ_LEN+8*3}>(&vd);
    }
}