macro_rules! serialize {
    ($t: ty, $fun: ident, $enc: ident, $fut: ident) => {
        impl crate::backend::Encodable for $t {
            type Encoder<F>
            where
                F: crate::backend::FormatEncode,
            = F::$enc;
        }

        impl crate::AsyncSerialize for $t {
            type Future<'w, F, W>
            where
                F: 'w + crate::backend::FormatSerialize,
                W: 'w + crate::io::AsyncWrite + Unpin,
            = F::$fut<'w, W>;
        
            fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
            where
                F: crate::backend::FormatSerialize,
                W: crate::io::AsyncWrite + Unpin,
            {
                F::$fun(format, writer, self)
            }
        }
    };
}

macro_rules! deserialize {
    ($t: ty, $fun: ident, $dec: ident, $fut: ident) => {
        impl crate::backend::Decodable for $t
        {
            type Decoder<F>
            where
                F: crate::backend::FormatDecode,
            = F::$dec;
        }

        impl crate::AsyncDeserialize for $t
        {
            type Future<'r, F, R>
            where
                F: 'r + crate::backend::FormatDeserialize,
                R: 'r + crate::io::AsyncBufRead + Unpin,
            = F::$fut<'r, R>;

            fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
            where
                F: crate::backend::FormatDeserialize,
                R: crate::io::AsyncBufRead + Unpin,
            {
                F::$fun(format, reader)
            }
        }
    };
}

macro_rules! primitive_def {
    ($t:ty, $ser_fn:ident, $ser_enc: ident, $ser_fut: ident, $deser_fn: ident, $deser_dec: ident, $deser_fut: ident) => {
        serialize!($t, $ser_fn, $ser_enc, $ser_fut);
        deserialize!($t, $deser_fn, $deser_dec, $deser_fut);
    };
}