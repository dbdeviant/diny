#![cfg_attr(not(feature = "std"), no_std)]

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

mod common;

#[cfg(any(feature = "std", feature = "alloc"))]
mod test {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::string::String;

    use super::common::SEQ_LEN;
    use super::common::method::eq::*;

    #[test]
    fn can_serialize_empty_string() {
        test_serialize_exact::<String, SEQ_LEN>(String::new());
    }

    #[test]
    fn can_serialize_single_char_string() {
        test_serialize_exact::<String, {SEQ_LEN + 1}>("a".into());
    }

    #[test]
    fn can_serialize_multi_char_string() {
        test_serialize_exact::<String, {SEQ_LEN + 3}>("abc".into());
    }

    #[test]
    fn can_serialize_long_string() {
        test_serialize_exact::<String, {SEQ_LEN + 26}>("abcdefghijklmnopqrstuvwxyz".into());
    }

    #[test]
    fn can_serialize_unicode_char_string() {
        test_serialize_exact::<String, {SEQ_LEN + 4}>("🔥".into());
    }
}