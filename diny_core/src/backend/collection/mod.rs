#[macro_use]
mod macros;

#[cfg(any(feature = "std", feature = "alloc"))]
mod vec {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::vec::Vec;
    use super::macros;
    
    impl<T> macros::SeqApi<T> for Vec<T> {
        type Iter<'a>
        where
            T: 'a,
        = core::iter::Skip<core::slice::Iter<'a, T>>;

        fn new() -> Self {
            Self::new()
        }

        fn reserve(&mut self, len: usize) {
            self.reserve_exact(len);
        }

        fn append(&mut self, t: T) {
            self.push(t);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    seq_collection_def!(Vec<T>);
}

#[cfg(feature = "std")]
mod vec_deque {
    use std::collections::VecDeque;
    use super::macros;

    impl<T> macros::SeqApi<T> for VecDeque<T> {
        type Iter<'a>
        where
            T: 'a,
        = std::iter::Skip<std::collections::vec_deque::Iter<'a, T>>;

        fn new() -> Self {
            Self::new()
        }

        fn reserve(&mut self, len: usize) {
            self.reserve_exact(len);
        }

        fn append(&mut self, t: T) {
            self.push_back(t);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    seq_collection_def!(VecDeque<T>);
}

#[cfg(feature = "std")]
mod linked_list {
    use std::collections::LinkedList;
    use super::macros;
    
    impl<T> macros::SeqApi<T> for LinkedList<T> {
        type Iter<'a>
        where
            T: 'a,
        = std::iter::Skip<std::collections::linked_list::Iter<'a, T>>;

        fn new() -> Self {
            Self::new()
        }

        fn reserve(&mut self, _len: usize) {
        }

        fn append(&mut self, t: T) {
            self.push_back(t);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    seq_collection_def!(LinkedList<T>);
}

#[cfg(feature = "std")]
mod binary_heap {
    use std::collections::BinaryHeap;
    use super::macros;
    
    impl<T> macros::SeqApi<T> for BinaryHeap<T>
    where
        T: Ord,
    {
        type Iter<'a>
        where
            T: 'a,
        = std::iter::Skip<std::collections::binary_heap::Iter<'a, T>>;

        fn new() -> Self {
            Self::new()
        }

        fn reserve(&mut self, _len: usize) {
        }

        fn append(&mut self, t: T) {
            self.push(t);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    seq_collection_def!(BinaryHeap<T: Ord>);
}

#[cfg(feature = "std")]
mod btree_set {
    use std::collections::{btree_set, BTreeSet};
    use super::macros;
    
    impl<T> macros::SeqApi<T> for BTreeSet<T>
    where
        T: Ord,
    {
        type Iter<'a>
        where
            T: 'a,
        = std::iter::Skip<btree_set::Iter<'a, T>>;

        fn new() -> Self {
            Self::new()
        }

        fn reserve(&mut self, _len: usize) {
        }

        fn append(&mut self, t: T) {
            self.insert(t);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    seq_collection_def!(BTreeSet<T: Ord>);
}


#[cfg(feature = "std")]
mod hash_set {
    use core::hash::{Hash, BuildHasher};
    use std::collections::{hash_set, HashSet};
    use super::macros;
    
    impl<T, S> macros::SeqApi<T> for HashSet<T, S>
    where
        T: Eq + Hash,
        S: BuildHasher + Default,
    {
        type Iter<'a>
        where
            T: 'a,
        = std::iter::Skip<hash_set::Iter<'a, T>>;

        fn new() -> Self {
            Self::with_hasher(S::default())
        }

        fn reserve(&mut self, len: usize) {
            self.reserve(len)
        }

        fn append(&mut self, t: T) {
            self.insert(t);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    seq_collection_def!(HashSet<T: Eq + Hash, S: BuildHasher + Default>);
}

#[cfg(feature = "std")]
mod btree_map {
    use std::collections::{btree_map, BTreeMap};
    use super::macros;
    
    impl<K, V> macros::MapApi<K, V> for BTreeMap<K, V>
    where
        K: Ord,
    {
        type Iter<'a>
        where
            K: 'a,
            V: 'a,
        = std::iter::Skip<btree_map::Iter<'a, K, V>>;

        fn new() -> Self {
            Self::new()
        }

        fn reserve(&mut self, _len: usize) {
        }

        fn append(&mut self, key: K, value: V) {
            self.insert(key, value);
        }

        fn iter_from(&self, idx: usize) -> Self::Iter<'_> {
            self.iter().skip(idx)
        }
    }

    map_collection_def!(BTreeMap<K: Ord, V>);
}