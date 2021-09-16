use super::*;
    
pub trait Iterable {
    type Item;
    type Iter<'a>: 'a + Iterator<Item=&'a Self::Item>
    where
        Self::Item: 'a
    ;

    fn iter(&self) -> Self::Iter<'_>;
}

pub fn cmp_ord<'a, I, T>(i1: &'a I, i2: &'a I) -> bool
where
    I: Iterable<Item = T>,
    T: 'a + PartialEq + core::fmt::Debug,
{
    let iter1 = i1.iter();
    let mut iter2 = i2.iter();
    for v1 in iter1 {
        if let Some(v2) = iter2.next() {
            if cmp_eq(&v1, &v2) {
                continue;
            }
        }
        return false;
    }
    iter2.next().is_none()
}

#[allow(unused)]
pub fn test_serialize<'a, C>(send: &'a C)
where
    C: diny::AsyncSerialize + diny::AsyncDeserialize + Iterable,
    C::Item: 'a + diny::AsyncSerialize + diny::AsyncDeserialize + PartialEq + core::fmt::Debug,
{
    #[cfg(any(feature = "std", feature = "alloc"))]
    assert!(cmp_ord(send, &serialize_vec(send)));
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    assert!(cmp_ord(send, &serialize_slice(send, &mut [0u8; 1024])));

    #[cfg(feature = "std")]
    assert!(cmp_ord(send, &serialize_pin_hole(send)));
}

#[allow(unused)]
pub fn test_serialize_exact<'a, C, const LEN: usize>(send: &'a C)
where
    C: diny::AsyncSerialize + diny::AsyncDeserialize + Iterable,
    C::Item: 'a + core::fmt::Debug + PartialEq + diny::AsyncSerialize + diny::AsyncDeserialize,
{
    assert!(cmp_ord(send, &serialize_exact::<C, LEN>(send)));

    #[cfg(feature = "std")]
    assert!(cmp_ord(send, &serialize_pin_hole(send)));
}

#[allow(unused)]
pub fn test_serialize_exact_with_error<T, const LEN: usize>(send: &T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
{
    serialize_slice_err(send, &mut [0u8; LEN]);
}