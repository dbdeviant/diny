#![feature(generic_associated_types)]

mod common;

use std::cell::RefCell;
use common::method::eq::*;

#[cfg(feature = "std")]
#[test]
fn can_serialize_ref_cell() {
    const LEN: usize = 8;
    test_serialize_exact::<RefCell<u64>, LEN>(RefCell::new(u64::MIN));
}

#[cfg(feature = "std")]
#[test]
fn can_serialize_borrowed_ref_cell() {
    const LEN: usize = 8;
    let ref_cell = RefCell::new(u64::MIN);
    let ref_borrow = ref_cell.borrow();
    test_serialize_exact_ref::<RefCell<u64>, LEN>(&ref_cell);
    let x = *ref_cell.borrow();
    assert_eq!(x, *ref_borrow)
}

#[cfg(feature = "std")]
#[test]
fn can_serialize_mut_borrowed_ref_cell_with_error() {
    const LEN: usize = 8;
    let ref_cell = RefCell::new(u64::MIN);
    let x;
    {
        let ref_borrow_mut = ref_cell.borrow_mut();
        test_serialize_exact_ref_with_error::<RefCell<u64>, LEN>(&ref_cell);
        x = *ref_borrow_mut;
    }
    let y = *ref_cell.borrow();
    assert_eq!(x, y)
}