macro_rules! map_collection_def {
    ($t: ident < K $(: $k_bound: ident $(+ $k_bounds: ident)*)?, V $(, $s: ident: $s_bound: ident $(+ $s_bounds: ident)*)? >) => {
        #[allow(unused)]
        use core::marker::PhantomData;
        use core::task::Context;
        use crate::{
            backend::{
                self,
                collection::macros::MapApi,
                Encode as _,
                Decode as _,
                internal::SequenceLen
            },
            io
        };


        type Data<K, V $(, $s)?> = $t<K, V $(, $s)?>;

        type Len = usize;
        type Idx = usize;

        pub enum Encoder<F, K, V $(, $s)?>
        where
            F: backend::FormatEncode,
            K: backend::Encodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Encodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            Init,
            Len(SequenceLen, <SequenceLen as backend::Encodable>::Encoder<F>),
            Key(Len, Idx, <K as backend::Encodable>::Encoder<F>),
            Val(Len, Idx, <V as backend::Encodable>::Encoder<F>),
            Fini,
            $(#[allow(dead_code)] Phantom(PhantomData<* const $s>))?
        }

        impl<F, K, V $(, $s)?> Encoder<F, K, V $(, $s)?>
        where
            F: backend::FormatEncode,
            K: backend::Encodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Encodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            #[allow(clippy::ptr_arg)]
            fn after_init<W>(format: &F, writer: &mut W, data: &Data<K, V $(, $s)?>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                let len: SequenceLen = data.len().into();
                match <SequenceLen as backend::Encodable>::Encoder::<F>::start_encode(format, writer, &len, cx) {
                    backend::StartEncodeStatus::Fini => Self::after_len(format, writer, *len, data, cx),
                    backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self::Len(len, enc)),
                    backend::StartEncodeStatus::Error(e) => backend::StartEncodeStatus::Error(e),
                }
            }

            #[allow(clippy::ptr_arg)]
            fn after_len<W>(format: &F, writer: &mut W, len: Len, data: &Data<K, V $(, $s)?>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                Self::items_from(format, writer, len, 0, <Data<K, V $(, $s)?> as MapApi<K, V>>::iter_from(data, 0), cx)
            }
                
            fn items_from<'a, W, I>(format: &F, writer: &mut W, len: usize, idx: usize, iter: I, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                K: 'a,
                V: 'a,
                I: Iterator<Item=(&'a K, &'a V)>,
                W: io::AsyncWrite + Unpin,
            {
                for (i, (k, v)) in iter.enumerate() {
                    match <K as backend::Encodable>::Encoder::<F>::start_encode(format, writer, k, cx) {
                        backend::StartEncodeStatus::Fini         => match <V as backend::Encodable>::Encoder::<F>::start_encode(format, writer, v, cx) {
                            backend::StartEncodeStatus::Fini         => continue,
                            backend::StartEncodeStatus::Pending(enc) => return backend::StartEncodeStatus::Pending(Self::Val(len, i + idx, enc)),
                            backend::StartEncodeStatus::Error(e)     => return backend::StartEncodeStatus::Error(e),
                        },
                        backend::StartEncodeStatus::Pending(enc) => return backend::StartEncodeStatus::Pending(Self::Key(len, i + idx, enc)),
                        backend::StartEncodeStatus::Error(e)     => return backend::StartEncodeStatus::Error(e),
                    }
                }

                backend::StartEncodeStatus::Fini
            }

            fn items_from_value<'a, W, I>(format: &F, writer: &mut W, len: usize, idx: usize, iter: I, v: &V, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                K: 'a,
                V: 'a,
                I: Iterator<Item=(&'a K, &'a V)>,
                W: io::AsyncWrite + Unpin,
            {
                match <V as backend::Encodable>::Encoder::<F>::start_encode(format, writer, v, cx) {
                    backend::StartEncodeStatus::Fini         => Self::items_from(format, writer, len, idx + 1, iter, cx),
                    backend::StartEncodeStatus::Pending(enc) => backend::StartEncodeStatus::Pending(Self::Val(len, idx, enc)),
                    backend::StartEncodeStatus::Error(e)     => backend::StartEncodeStatus::Error(e),
                }
            }
        }

        impl<F, K, V $(, $s)?> backend::Encode for Encoder<F, K, V $(, $s)?>
        where
            F: backend::FormatEncode,
            K: backend::Encodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Encodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Data = Data<K, V $(, $s)?>;
            type Format = F;

            fn init(_data: &Self::Data) -> Self {
                Self::Init
            }

            fn start_encode<W>(format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                Self::after_init(format, writer, data, cx)
            }

            fn poll_encode<W>(&mut self, format: &F, writer: &mut W, data: &Self::Data, cx: &mut Context<'_>) -> backend::PollEncodeStatus<<F as backend::Format>::Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                match self {
                    Self::Init               => encode_chain!(*self, Self::start_encode(format, writer, data, cx)),
                    Self::Len(len, enc)      => encode_poll_chain!(*self, enc.poll_encode(format, writer, len, cx), Self::after_len(format, writer, **len, data, cx)),
                    Self::Key(len, idx, enc) => {
                        let mut iter = <Self::Data as MapApi<K, V>>::iter_from(data, *idx);
                        match iter.next() {
                            Some((k, v)) => encode_poll_chain!(*self, enc.poll_encode(format, writer, k, cx), Self::items_from_value(format, writer, *len, *idx, iter, v, cx)),
                            None         => backend::PollEncodeStatus::Error(F::invalid_input_err()),
                        }                        
                    }, 
                    Self::Val(len, idx, enc) => {
                        let mut iter = <Self::Data as MapApi<K, V>>::iter_from(data, *idx);
                        match iter.next() {
                            Some((_, v)) => encode_poll_chain!(*self, enc.poll_encode(format, writer, v, cx), Self::items_from(format, writer, *len, *idx + 1, iter, cx)),
                            None         => backend::PollEncodeStatus::Error(F::invalid_input_err()),
                        }                        
                    }, 
                    _ => backend::PollEncodeStatus::Error(F::invalid_input_err()),
                }
            }
        }

        impl<K, V $(, $s)?> backend::Encodable for Data<K, V $(, $s)?>
        where
            K: backend::Encodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Encodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Encoder<F: backend::FormatEncode> = Encoder<F, K, V $(, $s)?>;
        }

        impl<K, V $(, $s)?> backend::AsyncSerialize for Data<K, V $(, $s)?>
        where
            K: backend::Encodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Encodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Future<'w, F, W>
            where
                Self: 'w,
                F: 'w + backend::FormatSerialize,
                W: 'w + io::AsyncWrite + Unpin,
            = backend::SerializeAll<'w, F, W, Self, Self::Encoder<F>>;

            fn serialize<'w, F, W>(&'w self, format: &'w F, writer: &'w mut W) -> Self::Future<'w, F, W>
            where
                F: backend::FormatSerialize,
                W: io::AsyncWrite + Unpin,

            {
                backend::SerializeAll::new(format, writer, self, <Self::Encoder::<F> as backend::Encode>::init(self))
            }
        }

        struct PartialData<K, V $(, $s)?> {
            map: Data<K, V $(, $s)?>,
            key: Option<K>,
        }

        impl<K $(: $k_bound $(+ $k_bounds)*)?, V $(, $s: $s_bound $(+ $s_bounds)*)?> PartialData<K, V $(, $s)?>
        {
            pub fn new() -> Self {
                Self {
                    map: <Data::<K, V $(, $s)?> as MapApi<K, V>>::new(),
                    key: None,
                }
            }

            pub fn into_data(self) -> Data<K, V $(, $s)?> {
                self.map
            }
        }

        enum DecodeCursor<F, K, V $(, $s)?>
        where
            F: backend::FormatDecode,
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            Init,
            Len(<SequenceLen as backend::Decodable>::Decoder<F>),
            Key(Len, Idx, <K as backend::Decodable>::Decoder<F>),
            Val(Len, Idx, <V as backend::Decodable>::Decoder<F>),
            Fini,
            $(#[allow(dead_code)] Phantom(PhantomData<* const $s>))?
        }

        struct DecodeState<F, K, V $(, $s)?>
        where
            F: backend::FormatDecode,
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            data: PartialData<K, V $(, $s)?>,
            cursor: DecodeCursor<F, K, V $(, $s)?>,
        }

        impl<F, K, V $(, $s)?> DecodeState<F, K, V $(, $s)?>
        where
            F: backend::FormatDecode,
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            pub fn new() -> Self {
                Self {
                    data: PartialData::new(),
                    cursor: DecodeCursor::Init,
                }
            }
        }

        impl<F, K, V $(, $s)?> DecodeCursor<F, K, V $(, $s)?>
        where
            F: backend::FormatDecode,
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData<K, V $(, $s)?>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncBufRead + Unpin,
            {
                <SequenceLen as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
                .and_then(
                    |len| Self::after_len(format, reader, *len, data, cx),
                    Self::Len,
                )
            }

            fn after_len<R>(format: &F, reader: &mut R, len: Len, data: &mut PartialData<K, V $(, $s)?>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncBufRead + Unpin,
            {
                <Data<K, V $(, $s)?> as MapApi<K, V>>::reserve(&mut data.map, len);
                Self::items_from(format, reader, len, 0, data, cx)
            }

            fn items_from<R>(format: &F, reader: &mut R, len: Len, idx: Idx, data: &mut PartialData<K, V $(, $s)?>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncBufRead + Unpin,
            {
                for i in idx..len {
                    match <K as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                        backend::StartDecodeStatus::Fini(k) => match <V as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                            backend::StartDecodeStatus::Fini(v)  => {
                                <Data<K, V $(, $s)?> as MapApi<K, V>>::append(&mut data.map, k, v);
                                continue
                            },
                            backend::StartDecodeStatus::Pending(dec) => {
                                data.key = Some(k);
                                return backend::StartDecodeStatus::Pending(Self::Val(len, i, dec))
                            }
                            backend::StartDecodeStatus::Error(e)     => return backend::StartDecodeStatus::Error(e),
                        },        
                        backend::StartDecodeStatus::Pending(dec) => return backend::StartDecodeStatus::Pending(Self::Key(len, i, dec)),
                        backend::StartDecodeStatus::Error(e)     => return backend::StartDecodeStatus::Error(e),
                    }
                }
        
                backend::StartDecodeStatus::Fini(())
            }

            fn items_from_value<R>(format: &F, reader: &mut R, len: Len, idx: Idx, data: &mut PartialData<K, V $(, $s)?>, k: K, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncBufRead + Unpin,
            {
                match <V as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                    backend::StartDecodeStatus::Fini(v)      => {
                        <Data<K, V $(, $s)?> as MapApi<K, V>>::append(&mut data.map, k, v);
                        Self::items_from(format, reader, len, idx + 1, data, cx)
                    },
                    backend::StartDecodeStatus::Pending(dec) => {
                        data.key = Some(k);
                        backend::StartDecodeStatus::Pending(Self::Val(len, idx, dec))
                    },
                    backend::StartDecodeStatus::Error(e)     => backend::StartDecodeStatus::Error(e),
                }
            }
        }

        pub struct Decoder<F, K, V $(, $s)?>
        where
            F: backend::FormatDecode,
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            state: Option<DecodeState<F, K, V $(, $s)?>>,
        }

        impl<F, K, V $(, $s)?> backend::Decode for Decoder<F, K, V $(, $s)?>
        where
            F: backend::FormatDecode,
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Data = Data<K, V $(, $s)?>;
            type Format = F;

            fn init() -> Self {
                Self { state: Some(DecodeState::new()) }
            }

            fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
            where
                R: io::AsyncBufRead + Unpin,
            {
                let mut data = PartialData::new();
                match DecodeCursor::after_init(format, reader, &mut data, cx) {
                    backend::StartDecodeStatus::Fini(())        => backend::StartDecodeStatus::Fini(data.into_data()),
                    backend::StartDecodeStatus::Pending(cursor) => backend::StartDecodeStatus::Pending(Self { state: Some(DecodeState { data, cursor }) }),
                    backend::StartDecodeStatus::Error(e)        => backend::StartDecodeStatus::Error(e),
                }
            }

            fn poll_decode<R>(&mut self, format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::PollDecodeStatus<Self::Data, <F as backend::Format>::Error>
            where
                R: io::AsyncBufRead + Unpin,
            {
                if let Some(state) = &mut self.state {
                    match &mut state.cursor {
                        DecodeCursor::Init => decode_chain!(state.cursor, DecodeCursor, DecodeCursor::after_init(format, reader, &mut state.data, cx)),
                        DecodeCursor::Len(dec) =>
                            decode_poll_chain!(
                                state.cursor,
                                DecodeCursor,
                                dec.poll_decode(format, reader, cx),
                                |len: SequenceLen| {
                                    DecodeCursor::after_len(format, reader, *len, &mut state.data, cx)
                                }
                            ),
                        DecodeCursor::Key(len, idx, dec) =>
                            decode_poll_chain!(
                                state.cursor,
                                DecodeCursor,
                                dec.poll_decode(format, reader, cx),
                                |k| {
                                    DecodeCursor::items_from_value(format, reader, *len, *idx, &mut state.data, k, cx)
                                }
                            ),
                        DecodeCursor::Val(len, idx, dec) =>
                            decode_poll_chain!(
                                state.cursor,
                                DecodeCursor,
                                dec.poll_decode(format, reader, cx),
                                |v| {
                                    // SAFETY:
                                    // The only way this code gets executured is if the key has already
                                    // been decoded and temporarily stored away in the partial data.
                                    // The only reason this is stored in the partial data as an option
                                    // is to appease the borrow checker.
                                    let key = state.data.key.take().unwrap();
                                    <Self::Data as MapApi<K, V>>::append(&mut state.data.map, key, v);
                                    DecodeCursor::items_from(format, reader, *len, *idx + 1, &mut state.data, cx)
                                }
                            ),
                        _ => return backend::PollDecodeStatus::Error(F::invalid_input_err()),
                    }
                    // SAFETY:
                    // The only way this code gets executed is if the outer state existed and reached
                    // the DecodeCursor::Fini state as a result of this call.  That cursor state is
                    // only reached once all array items have been created, and this next statement
                    // consumes the outer state in order to produce the returned array.
                    .map(|()| self.state.take().unwrap().data.into_data())
                } else {
                    backend::PollDecodeStatus::Error(F::invalid_input_err())
                }
            }
        }

        impl<K, V $(, $s)?> backend::Decodable for Data<K, V $(, $s)?>
        where
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Decoder<F: backend::FormatDecode> = Decoder<F, K, V $(, $s)?>;
        }

        impl<K, V $(, $s)?> backend::AsyncDeserialize for Data<K, V $(, $s)?>
        where
            K: backend::Decodable $(+ $k_bound $(+ $k_bounds)*)?,
            V: backend::Decodable,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Future<'r, F, R>
            where
                F: 'r + backend::FormatDeserialize,
                R: 'r + io::AsyncBufRead + Unpin,
            = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

            fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
            where
                F: backend::FormatDeserialize,
                R: io::AsyncBufRead + Unpin,
            {
                backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
            }
        }
    };
}