macro_rules! usize_wrapper_def {
    ($t:ident, $ser_fn:ident, $ser_enc: ident, $ser_fut: ident, $deser_fn: ident, $deser_dec: ident, $deser_fut: ident) => {
        use crate::backend::{Decodable, AsyncDeserialize, Encodable, FormatDecode, FormatDeserialize, FormatEncode, FormatSerialize, AsyncSerialize};

        #[repr(transparent)]
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $t(usize);
        
        impl $t {
            pub fn new(v: usize) -> Self {
                Self(v)
            }
        }
        
        impl From<usize> for $t {
            fn from(v: usize) -> Self {
                Self::new(v)
            }
        }
        
        impl From<$t> for usize {
            fn from(v: $t) -> Self {
                v.0
            }
        }
        
        impl core::ops::Deref for $t {
            type Target = usize;
        
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl core::ops::DerefMut for $t {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        
        impl Encodable for $t {
            type Encoder<F>
            where
                F: FormatEncode,
            = F::$ser_enc;
        }
        
        impl AsyncSerialize for $t {
            type Future<'w, F, W>
            where
                F: 'w + FormatSerialize,
                W: 'w + futures::AsyncWrite + Unpin,
            = F::$ser_fut<'w, W>;
        
            fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
            where
                F: crate::backend::FormatSerialize,
                W: futures::AsyncWrite + Unpin,
            {
                F::$ser_fn(format, writer, self)
            }
        }
        
        impl Decodable for $t {
            type Decoder<F>
            where
                F: FormatDecode,
            = F::$deser_dec;
        }
        
        impl AsyncDeserialize for $t {
            type Future<'r, F, R>
            where
                F: 'r + FormatDeserialize,
                R: 'r + ::futures::AsyncRead + ::futures::AsyncBufRead + Unpin,
            = F::$deser_fut<'r, R>;
        
            fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
            where
                F: FormatDeserialize,
                R: ::futures::AsyncRead + ::futures::AsyncBufRead + Unpin,
            {
                F::$deser_fn(format, reader)
            }
        }
    };
}