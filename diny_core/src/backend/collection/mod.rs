#[macro_use]
mod macros;

trait CollectionApi<T>
{
    type Iter<'a>: 'a + Iterator<Item=&'a T>
    where
        T: 'a
    ;

    fn new() -> Self;
    fn reserve(&mut self, len: usize);
    fn append(&mut self, value: T);
    fn iter_from(&self, idx: usize) -> Self::Iter<'_>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod vec {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::vec::Vec;
    use super::CollectionApi;
    
    impl<T> CollectionApi<T> for Vec<T> {
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
    use super::CollectionApi;

    impl<T> CollectionApi<T> for VecDeque<T> {
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
    use super::CollectionApi;
    
    impl<T> CollectionApi<T> for LinkedList<T> {
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
    use super::CollectionApi;
    
    impl<T> CollectionApi<T> for BinaryHeap<T>
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
    use super::CollectionApi;
    
    impl<T> CollectionApi<T> for BTreeSet<T>
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
    use super::CollectionApi;
    
    impl<T, S> CollectionApi<T> for HashSet<T, S>
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