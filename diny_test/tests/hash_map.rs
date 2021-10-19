#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use core::hash::{Hash, BuildHasher};
    use std::collections::{HashMap, HashSet};
    use super::common::SEQ_LEN;
    use super::common::method::set::*;

    impl<K, V, S> IsEquivalentTo for HashMap<K, V, S>
    where
        K: Eq + Hash,
        V: Eq + Hash,
        S: BuildHasher,
    {
        fn is_equivalent_to(&self, other: &Self) -> bool {
            let mut a = HashSet::new();
            for e in self {
                a.insert(e);
            }

            let mut b = HashSet::new();
            for e in other {
                b.insert(e);
            }

            a.symmetric_difference(&b).count() == 0
        }
    }
    
    #[test]
    fn can_serialize_empty_hash_set() {
        test_serialize_exact::<HashMap<u64, bool>, {SEQ_LEN}>(HashMap::new());
    }
    
    #[test]
    fn can_serialize_hash_set_single() {
        let mut vd = HashMap::new();
        vd.insert(5, true);
        test_serialize_exact::<HashMap<u64, bool>, {SEQ_LEN+8+1}>(vd);
    }
    
    #[test]
    fn can_serialize_hash_set_multiple() {
        let mut vd = HashMap::new();
        vd.insert(1, true );
        vd.insert(2, false);
        vd.insert(3, true );
        test_serialize_exact::<HashMap<u64, bool>, {SEQ_LEN+(8+1)*3}>(vd);
    }
}