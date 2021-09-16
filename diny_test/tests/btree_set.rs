#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use super::common::SEQ_LEN;
    use super::common::method::eq::*;
    use std::collections::BTreeSet;
    
    #[test]
    fn can_serialize_empty_btree_set() {
        test_serialize_exact::<BTreeSet<u64>, {SEQ_LEN}>(&BTreeSet::new());
    }
    
    #[test]
    fn can_serialize_btree_set_single() {
        let mut vd = BTreeSet::new();
        vd.insert(5);
        test_serialize_exact::<BTreeSet<u64>, {SEQ_LEN + 8}>(&vd);
    }
    
    #[test]
    fn can_serialize_btree_set_multiple() {
        let mut vd = BTreeSet::new();
        vd.insert(1);
        vd.insert(2);
        vd.insert(3);
        test_serialize_exact::<BTreeSet<u64>, {SEQ_LEN+8*3}>(&vd);
    }
}
