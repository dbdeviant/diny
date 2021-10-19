#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod common;

#[cfg(feature = "std")]
mod test {
    use super::common::SEQ_LEN;
    use super::common::method::eq::*;
    use std::collections::BTreeMap;
    
    #[test]
    fn can_serialize_empty_btree_set() {
        test_serialize_exact::<BTreeMap<u64, bool>, {SEQ_LEN}>(BTreeMap::new());
    }
    
    #[test]
    fn can_serialize_btree_set_single() {
        let mut vd = BTreeMap::new();
        vd.insert(5, true);
        test_serialize_exact::<BTreeMap<u64, bool>, {SEQ_LEN + 8 + 1}>(vd);
    }
    
    #[test]
    fn can_serialize_btree_set_multiple() {
        let mut vd = BTreeMap::new();
        vd.insert(1, true );
        vd.insert(2, false);
        vd.insert(3, true );
        test_serialize_exact::<BTreeMap<u64, bool>, {SEQ_LEN+(8+1)*3}>(vd);
    }
}
