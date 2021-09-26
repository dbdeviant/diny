//   $($s: $s_bound $(+ $s_bounds)*,)?
//  $($s: $s_bound $(+ $s_bounds)*,)?

macro_rules! seq_collection_def {
    ($t: ident < T $(: $t_bound: ident $(+ $t_bounds: ident)*)? $(, $s: ident: $s_bound: ident $(+ $s_bounds: ident)*)? >) => {
        #[allow(unused)]
        use core::marker::PhantomData;
        use core::task::Context;
        use crate::{backend::{self, Encode as _, Decode as _, internal::SequenceLen}};
        use crate::io;


        type Data<T $(, $s)?> = $t<T $(, $s)?>;

        type Len = usize;
        type Idx = usize;

        pub enum Encode<F, T $(, $s)?>
        where
            F: backend::FormatEncode,
            T: backend::Encodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            Init,
            Len(SequenceLen, <SequenceLen as backend::Encodable>::Encoder<F>),
            Cur(Len, Idx, <T as backend::Encodable>::Encoder<F>),
            Fini,
            $(#[allow(dead_code)] Phantom(PhantomData<* const $s>))?
        }

        impl<F, T $(, $s)?> Encode<F, T $(, $s)?>
        where
            F: backend::FormatEncode,
            T: backend::Encodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            #[allow(clippy::ptr_arg)]
            fn after_init<W>(format: &F, writer: &mut W, data: &Data<T $(, $s)?>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
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
            fn after_len<W>(format: &F, writer: &mut W, len: Len, data: &Data<T $(, $s)?>, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                W: io::AsyncWrite + Unpin,
            {
                Self::items_from(format, writer, len, 0, <Data<T $(, $s)?> as CollectionApi<T>>::iter_from(data, 0), cx)
            }
                
            fn items_from<'a, W, I>(format: &F, writer: &mut W, len: usize, idx: usize, iter: I, cx: &mut Context<'_>) -> backend::StartEncodeStatus<Self, <F as backend::Format>::Error>
            where
                T: 'a,
                I: Iterator<Item=&'a T>,
                W: io::AsyncWrite + Unpin,
            {
                for (i, d) in iter.enumerate() {
                    match <T as backend::Encodable>::Encoder::<F>::start_encode(format, writer, d, cx) {
                        backend::StartEncodeStatus::Fini         => continue,
                        backend::StartEncodeStatus::Pending(enc) => return backend::StartEncodeStatus::Pending(Self::Cur(len, i + idx, enc)),
                        backend::StartEncodeStatus::Error(e)     => return backend::StartEncodeStatus::Error(e),
                    }
                }

                backend::StartEncodeStatus::Fini
            }
        }

        impl<F, T $(, $s)?> backend::Encode for Encode<F, T $(, $s)?>
        where
            F: backend::FormatEncode,
            T: backend::Encodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Data = Data<T $(, $s)?>;
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
                    Self::Cur(len, idx, enc) => {
                        let mut iter = <Self::Data as CollectionApi<T>>::iter_from(data, *idx);
                        match iter.next() {
                            Some(d) => encode_poll_chain!(*self, enc.poll_encode(format, writer, d, cx), Self::items_from(format, writer, *len, *idx + 1, iter, cx)),
                            None    => backend::PollEncodeStatus::Error(F::invalid_input_err()),
                        }                        
                    }, 
                    _ => backend::PollEncodeStatus::Error(F::invalid_input_err()),
                }
            }
        }

        impl<T $(, $s)?> backend::Encodable for Data<T $(, $s)?>
        where
            T: backend::Encodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Encoder<F: backend::FormatEncode> = Encode<F, T $(, $s)?>;
        }

        impl<T $(, $s)?> backend::AsyncSerialize for Data<T $(, $s)?>
        where
            T: backend::Encodable $(+ $t_bound $(+ $t_bounds)*)?,
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

        struct PartialData<T $(, $s)?>(Data<T $(, $s)?>);

        impl<T $(: $t_bound $(+ $t_bounds)*)? $(, $s: $s_bound $(+ $s_bounds)*)?> PartialData<T $(, $s)?>
        {
            pub fn new() -> Self {
                Self(<Data::<T $(, $s)?> as CollectionApi<T>>::new())
            }

            pub fn into_data(self) -> Data<T $(, $s)?> {
                self.0
            }
        }

        impl<T $(, $s)?> core::ops::Deref for PartialData<T $(, $s)?> {
            type Target = Data<T $(, $s)?>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T $(, $s)?> core::ops::DerefMut for PartialData<T $(, $s)?> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        enum DecodeCursor<F, T $(, $s)?>
        where
            F: backend::FormatDecode,
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            Init,
            Len(<SequenceLen as backend::Decodable>::Decoder<F>),
            Cur(Len, Idx, <T as backend::Decodable>::Decoder<F>),
            Fini,
            $(#[allow(dead_code)] Phantom(PhantomData<* const $s>))?
        }

        struct DecodeState<F, T $(, $s)?>
        where
            F: backend::FormatDecode,
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            data: PartialData<T $(, $s)?>,
            cursor: DecodeCursor<F, T $(, $s)?>,
        }

        impl<F, T $(, $s)?> DecodeState<F, T $(, $s)?>
        where
            F: backend::FormatDecode,
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            pub fn new() -> Self {
                Self {
                    data: PartialData::new(),
                    cursor: DecodeCursor::Init,
                }
            }
        }

        impl<F, T $(, $s)?> DecodeCursor<F, T $(, $s)?>
        where
            F: backend::FormatDecode,
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            fn after_init<R>(format: &F, reader: &mut R, data: &mut PartialData<T $(, $s)?>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                <SequenceLen as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx)
                .and_then(
                    |len| Self::after_len(format, reader, *len, data, cx),
                    Self::Len,
                )
            }

            fn after_len<R>(format: &F, reader: &mut R, len: Len, data: &mut PartialData<T $(, $s)?>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                <Data<T $(, $s)?> as CollectionApi<T>>::reserve(data, len);
                Self::items_from(format, reader, len, 0, data, cx)
            }

            fn items_from<R>(format: &F, reader: &mut R, len: Len, idx: Idx, data: &mut PartialData<T $(, $s)?>, cx: &mut Context<'_>) -> backend::StartDecodeStatus<(), Self, <F as backend::Format>::Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                for i in idx..len {
                    match <T as backend::Decodable>::Decoder::<F>::start_decode(format, reader, cx) {
                        backend::StartDecodeStatus::Fini(d) => { <Data<T $(, $s)?> as CollectionApi<T>>::append(data, d); continue },
                        backend::StartDecodeStatus::Pending(dec) => return backend::StartDecodeStatus::Pending(Self::Cur(len, i, dec)),
                        backend::StartDecodeStatus::Error(e) => return backend::StartDecodeStatus::Error(e),
                    }
                }
        
                backend::StartDecodeStatus::Fini(())
            }
        }

        pub struct Decode<F, T $(, $s)?>
        where
            F: backend::FormatDecode,
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            state: Option<DecodeState<F, T $(, $s)?>>,
        }

        impl<F, T $(, $s)?> backend::Decode for Decode<F, T $(, $s)?>
        where
            F: backend::FormatDecode,
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Data = Data<T $(, $s)?>;
            type Format = F;

            fn init() -> Self {
                Self { state: Some(DecodeState::new()) }
            }

            fn start_decode<R>(format: &F, reader: &mut R, cx: &mut Context<'_>) -> backend::StartDecodeStatus<Self::Data, Self, <F as backend::Format>::Error>
            where
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
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
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
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
                        DecodeCursor::Cur(len, idx, dec) =>
                            decode_poll_chain!(
                                state.cursor,
                                DecodeCursor,
                                dec.poll_decode(format, reader, cx),
                                |d| {
                                    <Self::Data as CollectionApi<T>>::append(&mut state.data, d);
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

        impl<T $(, $s)?> backend::Decodable for Data<T $(, $s)?>
        where
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Decoder<F: backend::FormatDecode> = Decode<F, T $(, $s)?>;
        }

        impl<T $(, $s)?> backend::AsyncDeserialize for Data<T $(, $s)?>
        where
            T: backend::Decodable $(+ $t_bound $(+ $t_bounds)*)?,
            $($s: $s_bound $(+ $s_bounds)*,)?
        {
            type Future<'r, F, R>
            where
                F: 'r + backend::FormatDeserialize,
                R: 'r + io::AsyncRead + io::AsyncBufRead + Unpin,
            = backend::DeserializeExact<'r, F, R, Self, Self::Decoder<F>>;

            fn deserialize<'r, F, R>(format: &'r F, reader: &'r mut R) -> Self::Future<'r, F, R>
            where
                F: backend::FormatDeserialize,
                R: io::AsyncRead + io::AsyncBufRead + Unpin,
            {
                backend::DeserializeExact::new(format, reader, <Self::Decoder::<F> as backend::Decode>::init())
            }
        }
    };
}
