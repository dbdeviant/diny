use super::*;
    
pub trait IsEquivalentTo {
    fn is_equivalent_to(&self, other: &Self) -> bool;
}

#[allow(unused)]
pub fn test_serialize<C>(send: &C)
where
    C: diny::AsyncSerialize + diny::AsyncDeserialize + IsEquivalentTo,
{
    #[cfg(any(feature = "std", feature = "alloc"))]
    assert!(send.is_equivalent_to(&serialize_vec(send)));
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    assert!(send.is_equivalent_to(&serialize_slice(send, &mut [0u8; 1024])));

    #[cfg(feature = "std")]
    assert!(send.is_equivalent_to(&serialize_pin_hole(send)));
}

#[allow(unused)]
pub fn test_serialize_exact<C, const LEN: usize>(send: &C)
where
    C: diny::AsyncSerialize + diny::AsyncDeserialize + IsEquivalentTo,
{
    assert!(send.is_equivalent_to(&serialize_exact::<C, LEN>(send)));

    #[cfg(feature = "std")]
    assert!(send.is_equivalent_to(&serialize_pin_hole(send)));
}

#[allow(unused)]
pub fn test_serialize_exact_with_error<T, const LEN: usize>(send: &T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize,
{
    serialize_slice_err(send, &mut [0u8; LEN]);
}