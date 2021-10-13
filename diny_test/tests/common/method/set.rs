use super::*;
    
pub trait IsEquivalentTo {
    fn is_equivalent_to(&self, other: &Self) -> bool;
}

#[allow(unused)]
pub fn test_serialize<C>(send: C)
where
    C: diny::AsyncSerialization + IsEquivalentTo,
{
    #[cfg(any(feature = "std", feature = "alloc"))]
    let recv = serialize_vec(&send);
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    let recv = serialize_slice(send, &mut [0u8; 1024]);
    assert!(send.is_equivalent_to(&serialize_vec(&send)));

    #[cfg(feature = "std")]
    assert!(send.is_equivalent_to(&serialize_pin_hole(&send)));

    assert!(recv.is_equivalent_to(&stream(send)));
}

#[allow(unused)]
pub fn test_serialize_exact<C, const LEN: usize>(send: C)
where
    C: diny::AsyncSerialization + IsEquivalentTo,
{
    let recv = test_serialize_exact_ref::<C, LEN>(&send);
    assert!(recv.is_equivalent_to(&stream_exact::<C, LEN>(send)));
}

#[allow(unused)]
pub fn test_serialize_exact_ref<C, const LEN: usize>(send: &C) -> C
where
    C: diny::AsyncSerialization + IsEquivalentTo,
{
    let recv = serialize_exact_ref::<C, LEN>(send);
    assert!(send.is_equivalent_to(&recv));

    #[cfg(feature = "std")]
    assert!(send.is_equivalent_to(&serialize_pin_hole(send)));

    recv
}

#[allow(unused)]
pub fn test_serialize_exact_with_error<T, const LEN: usize>(send: &T)
where
    T: diny::AsyncSerialization,
{
    serialize_slice_err(send, &mut [0u8; LEN]);
}