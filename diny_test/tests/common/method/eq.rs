use super::*;
    

#[allow(unused)]
pub fn test_serialize<T>(send: T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + PartialEq + core::fmt::Debug,
{
    test_serialize_ref(&send);
    stream(send);

}

#[allow(unused)]
pub fn test_serialize_ref<T>(send: &T)
where
    T: diny::AsyncSerialize + diny::AsyncDeserialize + PartialEq + core::fmt::Debug,
{
    #[cfg(any(feature = "std", feature = "alloc"))]
    assert!(cmp_eq(send, &serialize_vec(send)));
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    assert!(cmp_ord(send, &serialize_slice(send, &mut [0u8; 1024])));

    #[cfg(feature = "std")]
    assert!(cmp_eq(send, &serialize_pin_hole(send)));
}

#[allow(unused)]
pub fn test_serialize_exact<T, const LEN: usize>(send: T)
where
    T: diny::AsyncSerialization + PartialEq + core::fmt::Debug,
{
    test_serialize_exact_ref::<T, LEN>(&send);
    stream_exact::<T, LEN>(send);
}

#[allow(unused)]
pub fn test_serialize_exact_ref<T, const LEN: usize>(send: &T)
where
    T: diny::AsyncSerialization + PartialEq + core::fmt::Debug,
{
    assert!(cmp_eq(send, &serialize_exact_ref::<T, LEN>(send)));

    #[cfg(feature = "std")]
    assert!(cmp_eq(send, &serialize_pin_hole(send)));
}

#[allow(unused)]
pub fn test_serialize_exact_ref_no_cmp<T, const LEN: usize>(send: &T) -> T
where
    T: diny::AsyncSerialization + PartialEq + core::fmt::Debug,
{
    serialize_exact_ref::<T, LEN>(send)
}

#[allow(unused)]
pub fn test_serialize_exact_ref_with_error<T, const LEN: usize>(send: &T)
where
    T: diny::AsyncSerialization,
{
    serialize_slice_err(send, &mut [0u8; LEN]);
}