#[macro_use]
pub mod macros;

#[cfg(any(feature = "std", feature = "alloc"))]
#[doc(hidden)] pub mod r#box {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::boxed::Box;
    
    wrapper_deref!(Box<T>);    
}

#[cfg(feature = "std")]
#[doc(hidden)] pub mod cell;
#[cfg(feature = "std")]
#[doc(hidden)] pub mod ref_cell;

#[cfg(feature = "std")]
#[doc(hidden)] pub mod rc { wrapper_deref!(::std::rc::Rc<T>); }
#[cfg(feature = "std")]
#[doc(hidden)] pub mod arc { wrapper_deref!(::std::sync::Arc<T>); }
