#[macro_use]
mod macros;

trait CollectionApi<T>
{
    type Iter<'a>: 'a + Iterator<Item=&'a T>
    where
        T: 'a
    ;

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