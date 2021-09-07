macro_rules! newtype {
    ($v:vis $newtype: ident($inner: ty))                     => { newtype_transparent_own!     ($v $newtype($inner));       };
    ($v:vis $newtype: ident<$lf: lifetime>(&$inner: ty))     => { newtype_transparent_ref!     ($v $newtype<$lf>(&$inner)); };
    ($v:vis $newtype: ident<$lf: lifetime>($inner: ty))      => { newtype_transparent_own_life!($v $newtype<$lf>($inner));  };
}

macro_rules! newtype_transparent_own {
    ($v:vis $newtype: ident ($inner: ty)) => {
        $v struct $newtype($inner);

        impl ::core::ops::Deref for $newtype {
            type Target = $inner;
        
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl ::core::ops::DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }   

        impl ::core::convert::AsRef<$inner> for $newtype {
            fn as_ref(&self) -> &$inner {
                self
            }
        }
        
        impl ::core::convert::AsMut<$inner> for $newtype {
            fn as_mut(&mut self) -> &mut $inner {
                self
            }
        }
        
        impl ::core::borrow::Borrow<$inner> for $newtype {
            fn borrow(&self) -> &$inner {
                self
            }
        }

        impl ::core::borrow::BorrowMut<$inner> for $newtype {
            fn borrow_mut(&mut self) -> &mut $inner {
                self
            }
        }

        impl From<$inner> for $newtype {
            fn from(v: $inner) -> Self {
                Self(v)
            }
        }

        impl From<$newtype> for $inner {
            fn from(v: $newtype) -> Self {
                v.0
            }
        }
    };
}

macro_rules! newtype_transparent_own_life {
    ($v:vis $newtype: ident<$lf: lifetime>($inner: ty)) => {
        $v struct $newtype<$lf>($inner);

        impl<$lf> ::core::ops::Deref for $newtype<$lf> {
            type Target = $inner;
        
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl<$lf> ::core::ops::DerefMut for $newtype<$lf> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }   

        impl<$lf> ::core::convert::AsRef<$inner> for $newtype<$lf> {
            fn as_ref(&self) -> &$inner {
                self
            }
        }
        
        impl<$lf> ::core::convert::AsMut<$inner> for $newtype<$lf> {
            fn as_mut(&mut self) -> &mut $inner {
                self
            }
        }
        
        impl<$lf> ::core::borrow::Borrow<$inner> for $newtype<$lf> {
            fn borrow(&self) -> &$inner {
                self
            }
        }

        impl<$lf> ::core::borrow::BorrowMut<$inner> for $newtype<$lf> {
            fn borrow_mut(&mut self) -> &mut $inner {
                self
            }
        }

        impl<$lf> From<$inner> for $newtype<$lf> {
            fn from(v: $inner) -> Self {
                Self(v)
            }
        }

        impl<$lf> From<$newtype<$lf>> for $inner {
            fn from(v: $newtype<$lf>) -> Self {
                v.0
            }
        }
    };
}

macro_rules! newtype_transparent_ref {
    ($v:vis $newtype: ident<$lf: lifetime>(&$inner: ty)) => {
        $v struct $newtype<$lf>(&$lf $inner);

        impl core::ops::Deref for $newtype<'_> {
            type Target = $inner;
        
            fn deref(&self) -> &Self::Target {
                self.0
            }
        }

        impl ::core::convert::AsRef<$inner> for $newtype<'_> {
            fn as_ref(&self) -> &$inner {
                self
            }
        }
        
        impl ::core::borrow::Borrow<$inner> for $newtype<'_> {
            fn borrow(&self) -> &$inner {
                self
            }
        }

        impl<$lf> From<&$lf $inner> for $newtype<$lf> {
            fn from(v: &$lf $inner) -> Self {
                Self(v)
            }
        }

        impl<$lf> From<$newtype<$lf>> for &$lf $inner {
            fn from(v: $newtype<$lf>) -> Self {
                v.0
            }
        }
    };
}