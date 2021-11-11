#[macro_use]
pub mod map;
#[macro_use]
pub mod seq;


pub trait MapApi<K, V>
{
    type Iter<'a>: 'a + Iterator<Item=(&'a K, &'a V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a,
    ;

    fn new() -> Self;
    fn reserve(&mut self, len: usize);
    fn append(&mut self, key: K, value: V);
    fn iter_from(&self, idx: usize) -> Self::Iter<'_>;
}

pub trait SeqApi<T>
{
    type Iter<'a>: 'a + Iterator<Item=&'a T>
    where
        Self: 'a,
        T: 'a
    ;

    fn new() -> Self;
    fn reserve(&mut self, len: usize);
    fn append(&mut self, value: T);
    fn iter_from(&self, idx: usize) -> Self::Iter<'_>;
}