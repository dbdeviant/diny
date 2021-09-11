use proc_macro2::TokenStream;
use quote::ToTokens;
use crate::data;
use super::encoded_field_gen::*;
use super::encoded_variant_gen::*;


pub fn generate_async_serialize(def: &data::Def) -> TokenStream {
    let type_name = def.name.to_token_stream();
    match &def.shape {
        data::Shape::Enum  (vs) => gen_enum_serialize(&type_name, vs),
        data::Shape::Struct(fs) => gen_struct_serialize(&type_name, fs),
    }
}

pub fn generate_async_deserialize(def: &data::Def) -> TokenStream {
    let type_name = def.name.to_token_stream();
    match &def.shape {
        data::Shape::Enum  (vs) => gen_enum_deserialize(&type_name, vs),
        data::Shape::Struct(fs) => gen_struct_deserialize(&type_name, fs),
    }
}

fn gen_struct_serialize(type_name: &TokenStream, fs: &data::Fields) -> TokenStream {

    fn non_empty_fields(fs: &data::Fields) -> TokenStream {
        let encoded_fields: EncodedFieldsGen = fs.into();
        let variants = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let type_ref = &field.field.type_ref;
            
            let err_src = syn::spanned::Spanned::span(&type_ref);
            quote_spanned! {err_src=> #ctor(<#type_ref as ::diny::backend::Encodable>::Encoder::<__F>) }
        });

        let methods = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let field_name = &field.field.id.field_name();
            let type_ref = &field.field.type_ref;
            let this_method = &field.this_method;

            let next = &field.next_method.as_ref().map_or_else(
                ||  quote! { ::core::result::Result::Ok(Self::Fini) },
                |n| quote! { Self::#n(format, writer, data, cx) },
            );

            quote! {
                fn #this_method<__W>(format: &__F, writer: &mut __W, data: &__Data, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    <<#type_ref as ::diny::backend::Encodable>::Encoder::<__F> as ::diny::backend::Encode>::start_encode(format, writer, &data.#field_name, cx)
                    .and_then(|o| match o {
                        ::core::option::Option::None    => #next,
                        ::core::option::Option::Some(s) => ::core::result::Result::Ok(Self::#ctor(s)),
                    })
                }
            }
        });

        let transitions = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let field_name = &field.field.id.field_name();

            let next = &field.next_method.as_ref().map_or_else(
                ||  quote! { ::core::result::Result::Ok(Self::Fini) },
                |n| quote! { Self::#n(format, writer, data, cx) },
            );

            quote! {
                Self::#ctor(enc) => {
                    ::futures::ready!(enc.poll_encode(format, writer, &data.#field_name, cx))
                    .and_then(|_| #next)
                }
            }
        });

        quote! {
            pub enum __Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            {
                Init,
                #(#variants,)*
                Fini,
            }

            impl<__F> __Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            {
                #(#methods)*
            }                
        
            impl<__F> ::diny::backend::Encode for __Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            {
                type Data = __Data;
                type Format = __F;
        
                fn init(data: &Self::Data) -> Self {
                    Self::Init
                }

                fn start_encode<__W>(format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::core::option::Option<Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    Self::after_init(format, writer, data, cx)
                    .map(|s| match s {
                        Self::Fini => ::core::option::Option::None,
                        _          => ::core::option::Option::Some(s),
                    })
                }
        
                fn poll_encode<__W>(&mut self, format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<::core::result::Result<(), <<Self as ::diny::backend::Encode>::Format as ::diny::backend::Format>::Error>>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    let res = match self {
                        Self::Init => {
                            Self::after_init(format, writer, data, cx)
                        },
                        #(#transitions)*
                        Self::Fini => {
                            ::core::result::Result::Err(__F::invalid_input_err())
                        }
                    };
        
                    match res {
                        ::core::result::Result::Ok(enc) => {
                            *self = enc;
                            match self {
                                Self::Fini => ::core::task::Poll::Ready(::core::result::Result::Ok(())),
                                _          => ::core::task::Poll::Pending,
                            }
                        },
                        ::core::result::Result::Err(e) => {
                            *self = Self::Fini;
                            ::core::task::Poll::Ready(::core::result::Result::Err(e))
                        }
                    }
                }
            }
        }
    }

    let (
        encode_impl,
        encode_type,
        encode_init
    ) =
        if fs.len() > 0 {
            (
                non_empty_fields(fs),
                quote! { __Encoder<__F> },
                quote! { Self::Encoder::<__F>::Init }
            )
        } else {
            (
                quote! {},
                quote! { ::diny::backend::intrinsic::empty_struct::Encoder::<__F, Self> },
                quote! { ::diny::backend::intrinsic::empty_struct::Encoder::<__F, Self>::init() }
            )
        };

    quote! {
        type __Data = #type_name;

        #encode_impl

        impl ::diny::backend::Encodable for __Data {
            type Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            = #encode_type;
        }
    
        impl ::diny::backend::AsyncSerialize for __Data {
            type Future<'w, __F, __W>
            where
                __F: 'w + ::diny::backend::FormatSerialize,
                __W: 'w + ::futures::io::AsyncWrite + ::core::marker::Unpin,
                = ::diny::backend::SerializeAll<'w, __F, __W, Self, Self::Encoder<__F>>;
    
            fn serialize<'w, __F, __W>(&'w self, format: &'w __F, writer: &'w mut __W) -> Self::Future<'w, __F, __W>
            where
                __F: ::diny::backend::FormatSerialize,
                __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
            {
                ::diny::backend::SerializeAll::new(format, writer, self, #encode_init)
            }
        }            
    }
}

fn gen_struct_deserialize(type_name: &TokenStream, fs: &data::Fields) -> TokenStream {

    fn empty_fields() -> TokenStream {
        quote! {
            impl ::diny::backend::intrinsic::empty_struct::NewUnitStruct for __Data {
                fn new_unit_struct() -> Self {
                    __Data {}
                }
            }
        }
    }

    fn non_empty_fields(fs: &data::Fields) -> TokenStream {
        let encoded_fields: EncodedFieldsGen = fs.into();

        let partial_named_fields = encoded_fields.iter().map(|field| {
            let name = &field.field.id.field_name();
            let type_ref = &field.field.type_ref;

            quote! { #name: ::core::option::Option<#type_ref> }
        });

        let partial_unnamed_fields = encoded_fields.iter().map(|field| {
            let type_ref = &field.field.type_ref;

            quote! { ::core::option::Option<#type_ref> }
        });

        let partial_field_defaults = encoded_fields.iter().map(|field| {
            let name = &field.field.id.field_name();

            quote! { #name: ::core::option::Option::None }
        });

        let partial_field_assignments = encoded_fields.iter().map(|field| {
            let name = &field.field.id.field_name();

            quote! { #name: self.#name? }
        });

        let variants = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let type_ref = &field.field.type_ref;
            
            let err_src = syn::spanned::Spanned::span(&type_ref);
            quote_spanned! {err_src=> #ctor(<#type_ref as ::diny::backend::Decodable>::Decoder::<__F>) }
        });

        let methods = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let field_name = &field.field.id.field_name();
            let type_ref = &field.field.type_ref;
            let this_method = &field.this_method;

            let next = &field.next_method.as_ref().map_or_else(
                ||  quote! { ::core::result::Result::Ok(Self::Fini) },
                |n| quote! { Self::#n(format, reader, data, cx) },
            );
            
            quote! {
                fn #this_method<__R>(format: &__F, reader: &mut __R, data: &mut __PartialData, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    <<#type_ref as ::diny::backend::Decodable>::Decoder::<__F> as ::diny::backend::Decode>::start_decode(format, reader, cx)
                    .and_then(|s| match s {
                        ::diny::backend::DecodeStatus::Ready  (d) => { data.#field_name = ::core::option::Option::Some(d); #next },
                        ::diny::backend::DecodeStatus::Pending(p) => ::core::result::Result::Ok(Self::#ctor(p)),
                    })
                }
            }
        });

        let transitions = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let field_name = &field.field.id.field_name();

            let next = &field.next_method.as_ref().map_or_else(
                ||  quote! { ::core::result::Result::Ok(__DecodeCursor::<__F>::Fini) },
                |n| quote! { __DecodeCursor::#n(format, reader, &mut state.data, cx) },
            );

            quote! {
                __DecodeCursor::#ctor(dec) => {
                    ::futures::ready!(dec.poll_decode(format, reader, cx))
                    .and_then(|d| { state.data.#field_name = ::core::option::Option::Some(d); #next })
                }
            }
        });

        let partial_field_def = match fs.style() {
            data::FieldsStyle::Named => quote! { { #(#partial_named_fields,)* } },
            data::FieldsStyle::Unnamed => quote! { ( #(#partial_unnamed_fields,)* ); },
        };

        quote! {
            struct __PartialData #partial_field_def

            impl __PartialData {
                pub fn new() -> Self {
                    Self {
                        #(#partial_field_defaults,)*
                    }
                }

                pub fn into_data(&self) -> ::core::option::Option<__Data> {
                    ::core::option::Option::Some(__Data {
                        #(#partial_field_assignments,)*
                    })
                }
            }

            enum __DecodeCursor<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                Init,
                #(#variants,)*
                Fini,
            }

            impl<__F> __DecodeCursor<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                #(#methods)*
            }                
        
            struct __DecodeState<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                data: __PartialData,
                cursor: __DecodeCursor<__F>,
            }

            impl<__F> __DecodeState<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                pub fn new() -> Self {
                    Self {
                        data: __PartialData::new(),
                        cursor: __DecodeCursor::Init,
                    }
                }
            }

            pub struct __Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                state: ::core::option::Option<__DecodeState<__F>>,
            }

            impl<__F> __Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                pub fn new() -> Self {
                    Self {
                        state: ::core::option::Option::Some(__DecodeState::new()),
                    }
                }
            }

            impl<__F> ::diny::backend::Decode for __Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                type Data = __Data;
                type Format = __F;

                fn init() -> Self {
                    Self::new()
                }
        
                fn start_decode<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::diny::backend::DecodeStatus<Self::Data, Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    let mut data = __PartialData::new();
                    __DecodeCursor::after_init(format, reader, &mut data, cx)
                    .and_then(|cursor| match cursor {
                        __DecodeCursor::Fini => {
                            match data.into_data() {
                                ::core::option::Option::None    => ::core::result::Result::Err(__F::invalid_data_err()),  // Should be unreachable!()
                                ::core::option::Option::Some(d) => ::core::result::Result::Ok(::diny::backend::DecodeStatus::Ready(d)),
                            }
                        },
                        _  => ::core::result::Result::Ok(::diny::backend::DecodeStatus::Pending(Self { state: ::core::option::Option::Some(__DecodeState{ data, cursor }) })),
                    })
                }


                fn poll_decode<__R>(&mut self, format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<::core::result::Result<Self::Data, <__F as ::diny::backend::Format>::Error>>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    match &mut self.state {
                        ::core::option::Option::None => ::core::task::Poll::Ready(::core::result::Result::Err(__F::invalid_input_err())),
                        ::core::option::Option::Some(state) => {
                            let res = match &mut state.cursor {
                                __DecodeCursor::Init => {
                                    __DecodeCursor::<__F>::after_init(format, reader, &mut state.data, cx)
                                }
                                #(#transitions)*
                                __DecodeCursor::Fini => {
                                    ::core::result::Result::Err(__F::invalid_input_err())
                                }
                            };
                
                            let ret = match res {
                                ::core::result::Result::Ok(dec) => {
                                    state.cursor = dec;
                                    match state.cursor {
                                        __DecodeCursor::Fini => {
                                            let ret = state.data.into_data();
                                            match ret {
                                                ::core::option::Option::None    => ::core::task::Poll::Ready(::core::result::Result::Err(__F::invalid_data_err())), // Should be unreachable!()
                                                ::core::option::Option::Some(d) => ::core::task::Poll::Ready(::core::result::Result::Ok(d)),
                                            }
                                        },
                                        _  => return ::core::task::Poll::Pending,
                                    }
                                },
                                ::core::result::Result::Err(e) => {
                                    ::core::task::Poll::Ready(::core::result::Result::Err(e))
                                }
                            };

                            self.state = ::core::option::Option::None;
                            ret
                        }
                    }
                }
            }
        }
    }

    let (
        decode_impl,
        decode_type,
        decode_init
    ) =
        if fs.len() > 0 {
            (
                non_empty_fields(fs),
                quote! { __Decoder<__F> },
                quote! { <Self::Decoder::<__F> as ::diny::backend::Decode>::init() }
            )
        } else {
            (
                empty_fields(),
                quote! { ::diny::backend::intrinsic::empty_struct::Decoder::<__F, Self> },
                quote! { <::diny::backend::intrinsic::empty_struct::Decoder::<__F, Self> as ::diny::backend::Decode>::init() }
            )
        };

    quote! {
        type __Data = #type_name;

        #decode_impl

        impl ::diny::backend::Decodable for __Data {
            type Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            = #decode_type;
        }
    
        impl ::diny::backend::AsyncDeserialize for __Data {
            type Future<'r, __F, __R>
            where
                __F: 'r + ::diny::backend::FormatDeserialize,
                __R: 'r + ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
            = ::diny::backend::DeserializeExact<'r, __F, __R, Self, Self::Decoder::<__F>>;
    
            fn deserialize<'r, __F, __R>(format: &'r __F, reader: &'r mut __R) -> Self::Future<'r, __F, __R>
            where
                __F: ::diny::backend::FormatDeserialize,
                __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
            {
                ::diny::backend::DeserializeExact::new(format, reader, #decode_init)
            }
        }
    }
}

fn gen_enum_serialize(type_name: &TokenStream, vs: &data::Variants) -> TokenStream {

    fn non_empty_variants(vs: &data::Variants) -> TokenStream {
        let encoded_variants: EncodedVariantsGen = vs.into();

        let variants = encoded_variants.iter().map(|variant| {
            let encode_ctor = &variant.ctor;
            let type_ref = variant.type_ref.to_token_stream();

            let err_src = syn::spanned::Spanned::span(&type_ref);
            quote_spanned! {err_src=> #encode_ctor(<#type_ref as ::diny::backend::Encodable>::Encoder::<__F>) }
        });

        let variant_indices = encoded_variants.iter().enumerate().map(|(id, variant)| {
            let data_ctor = &variant.variant.ctor;

            match &variant.type_ref {
                VariantType::Unit       => quote! { __Data::#data_ctor{}  => #id },
                VariantType::TypeRef(_) => quote! { __Data::#data_ctor(_) => #id },
            }
        });

        let dispatch = encoded_variants.iter().map(|variant| {
            let data_ctor = &variant.variant.ctor;
            let this_method = &variant.this_method;

            match &variant.type_ref {
                VariantType::Unit       => quote! { __Data::#data_ctor{}  => Self::#this_method(format, writer, &(), cx) },
                VariantType::TypeRef(_) => quote! { __Data::#data_ctor(d) => Self::#this_method(format, writer, d  , cx) },
            }
        });

        let methods = encoded_variants.iter().map(|variant| {
            let encode_ctor = &variant.ctor;
            let type_ref = &variant.type_ref.to_token_stream();
            let this_method = &variant.this_method;

            quote! {
                fn #this_method<__W>(format: &__F, writer: &mut __W, data: &#type_ref, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    <<#type_ref as ::diny::backend::Encodable>::Encoder::<__F> as ::diny::backend::Encode>::start_encode(format, writer, &data, cx)
                    .map(|o| match o {
                        ::core::option::Option::None    => Self::Fini,
                        ::core::option::Option::Some(s) => Self::#encode_ctor(s),
                    })
                }
            }
        });

        let transitions = encoded_variants.iter().map(|variant| {
            let encode_ctor = &variant.ctor;
            let data_ctor = &variant.variant.ctor;

            let poll = match &variant.type_ref {
                VariantType::Unit       => quote! { __Data::#data_ctor{}  => ::futures::ready!(enc.poll_encode(format, writer, &(), cx)) },
                VariantType::TypeRef(_) => quote! { __Data::#data_ctor(d) => ::futures::ready!(enc.poll_encode(format, writer, d  , cx)) },
            };

            quote! {
                Self::#encode_ctor(enc) => {
                    match data {
                        #poll.map(|_| Self::Fini),
                        _ => { debug_assert!(false); Err(__F::invalid_input_err()) },
                    }
                }
            }
        });

        quote! {
            pub enum __Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            {
                Init,
                Index(::diny::backend::internal::VariantIdx, <::diny::backend::internal::VariantIdx as ::diny::backend::Encodable>::Encoder<__F>),
                #(#variants,)*
                Fini,
            }

            impl<__F> __Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            {
                fn variant_index(data: &__Data) -> ::diny::backend::internal::VariantIdx {
                    match data {
                        #(#variant_indices,)*
                    }.into()
                }
        
                fn after_init<__W>(format: &__F, writer: &mut __W, data: &__Data, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    let index = Self::variant_index(data);
                    <<::diny::backend::internal::VariantIdx as ::diny::backend::Encodable>::Encoder::<__F> as ::diny::backend::Encode>::start_encode(format, writer, &index, cx)
                    .and_then(|o| match o {
                        ::core::option::Option::Some(s) => ::core::result::Result::Ok(Self::Index(index, s)),
                        ::core::option::Option::None    => Self::after_index(format, writer, data, cx),
                    })
                }
                
                fn after_index<__W>(format: &__F, writer: &mut __W, data: &__Data, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    match data {
                        #(#dispatch,)*
                    }
                }        

                #(#methods)*
            }                
        
            impl<__F> ::diny::backend::Encode for __Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            {
                type Data = __Data;
                type Format = __F;
        
                fn init(data: &Self::Data) -> Self {
                    Self::Init
                }

                fn start_encode<__W>(format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::core::option::Option<Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    Self::after_init(format, writer, data, cx)
                    .map(|s| match s {
                        Self::Fini => ::core::option::Option::None,
                        _          => ::core::option::Option::Some(s),
                    })
                }
        
                fn poll_encode<__W>(&mut self, format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<::core::result::Result<(), <<Self as ::diny::backend::Encode>::Format as ::diny::backend::Format>::Error>>
                where
                    __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
                {
                    // Contract: 'data' must not be modified between calls
                    let res = match self {
                        Self::Init => {
                            Self::after_init(format, writer, data, cx)
                        }
                        Self::Index(index, enc) => {
                            debug_assert_eq!(*index, Self::variant_index(data));
                            ::futures::ready!(enc.poll_encode(format, writer, &index, cx))
                            .and_then(|_| Self::after_index(format, writer, data, cx))
                        }
                        #(#transitions)*
                        Self::Fini => {
                            debug_assert!(false);
                            ::core::result::Result::Err(__F::invalid_input_err())
                        }
                    };
        
                    match res {
                        ::core::result::Result::Ok(enc) => {
                            *self = enc;
                            match self {
                                Self::Fini => ::core::task::Poll::Ready(::core::result::Result::Ok(())),
                                _          => ::core::task::Poll::Pending,
                            }
                        },
                        ::core::result::Result::Err(e) => {
                            *self = Self::Fini;
                            ::core::task::Poll::Ready(::core::result::Result::Err(e))
                        }
                    }
                }
            }
        }
    }

    let (
        encode_impl,
        encode_type,
        encode_init
    ) =
        if vs.len() > 0 {
            (
                non_empty_variants(vs),
                quote! { __Encoder<__F> },
                quote! { Self::Encoder::<__F>::Init }
            )
        } else {
            (
                quote! {},
                quote! { ::diny::backend::intrinsic::empty_struct::Encoder::<__F, Self> },
                quote! { ::diny::backend::intrinsic::empty_struct::Encoder::<__F, Self>::init() }
            )
        };

    quote! {
        type __Data = #type_name;

        #encode_impl

        impl ::diny::backend::Encodable for __Data {
            type Encoder<__F>
            where
                __F: ::diny::backend::FormatEncode,
            = #encode_type;
        }
    
        impl ::diny::backend::AsyncSerialize for __Data {
            type Future<'w, __F, __W>
            where
                __F: 'w + ::diny::backend::FormatSerialize,
                __W: 'w + ::futures::io::AsyncWrite + ::core::marker::Unpin,
                = ::diny::backend::SerializeAll<'w, __F, __W, Self, Self::Encoder<__F>>;
    
            fn serialize<'w, __F, __W>(&'w self, format: &'w __F, writer: &'w mut __W) -> Self::Future<'w, __F, __W>
            where
                __F: ::diny::backend::FormatSerialize,
                __W: ::futures::io::AsyncWrite + ::core::marker::Unpin,
            {
                ::diny::backend::SerializeAll::new(format, writer, self, #encode_init)
            }
        }            
    }
}

fn gen_enum_deserialize(type_name: &TokenStream, vs: &data::Variants) -> TokenStream {

    fn non_empty_fields(vs: &data::Variants) -> TokenStream {
        let encoded_variants: EncodedVariantsGen = vs.into();

        let variants = encoded_variants.iter().map(|variant| {
            let decode_ctor = &variant.ctor;
            let type_ref = &variant.type_ref.to_token_stream();
            
            let err_src = syn::spanned::Spanned::span(&type_ref);
            quote_spanned! {err_src=> #decode_ctor(<#type_ref as ::diny::backend::Decodable>::Decoder::<__F>) }
        });

        let dispatch = encoded_variants.iter().enumerate().map(|(id, variant)| {
            let this_method = &variant.this_method;

            quote! { #id => Self::#this_method(format, reader, cx) }
        });

        let methods = encoded_variants.iter().map(|variant| {
            let data_ctor = &variant.variant.ctor;
            let decode_ctor = &variant.ctor;
            let type_ref = &variant.type_ref.to_token_stream();
            let this_method = &variant.this_method;

            let bimap_ctor = match &variant.type_ref {
                VariantType::Unit => quote! { |_| __Data::#data_ctor{} },
                VariantType::TypeRef(_) => quote! { __Data::#data_ctor },
            };

            quote! {
                fn #this_method<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::diny::backend::DecodeStatus<__Data, Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    <<#type_ref as ::diny::backend::Decodable>::Decoder::<__F> as ::diny::backend::Decode>::start_decode(format, reader, cx)
                    .map(|status| status.bimap(#bimap_ctor, Self::#decode_ctor))
                }
            }
        });

        let transitions = encoded_variants.iter().map(|variant| {
            let decode_ctor = &variant.ctor;
            let data_ctor = &variant.variant.ctor;
            let ctor = match &variant.type_ref {
                VariantType::Unit => quote! { #data_ctor{} },
                VariantType::TypeRef(_) => quote! { #data_ctor(_d) },
            };

            quote! {
                Self::#decode_ctor(dec) => {
                    ::futures::ready!(dec.poll_decode(format, reader, cx))
                    .map(|_d| ::diny::backend::DecodeStatus::Ready(__Data::#ctor))
                }
            }
        });

        quote! {
            pub enum __Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                Init,
                Index(<::diny::backend::internal::VariantIdx as ::diny::backend::Decodable>::Decoder<__F>),
                #(#variants,)*
                Fini,
            }

            impl<__F> __Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                fn from_index<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::diny::backend::DecodeStatus<__Data, Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    <<diny::backend::internal::VariantIdx as ::diny::backend::Decodable>::Decoder::<__F> as ::diny::backend::Decode>::start_decode(format, reader, cx)
                    .and_then(|status| match status {
                        ::diny::backend::DecodeStatus::Ready(idx) => Self::from_data(&idx, format, reader, cx),
                        ::diny::backend::DecodeStatus::Pending(p) => ::core::result::Result::Ok(::diny::backend::DecodeStatus::Pending(Self::Index(p))),
                    })
                }
        
                fn from_data<__R>(index: &::diny::backend::internal::VariantIdx, format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::diny::backend::DecodeStatus<__Data, Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    match **index {
                        #(#dispatch,)*
                        _ => ::core::result::Result::Err(__F::invalid_input_err()),
                    }
                }
                
                #(#methods)*
            }

            impl<__F> ::diny::backend::Decode for __Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            {
                type Data = __Data;
                type Format = __F;
        
                fn init() -> Self {
                    Self::Init
                }

                fn start_decode<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::result::Result<::diny::backend::DecodeStatus<Self::Data, Self>, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    Self::from_index(format, reader, cx)
                }


                fn poll_decode<__R>(&mut self, format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<::core::result::Result<Self::Data, <__F as ::diny::backend::Format>::Error>>
                where
                    __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    let res = match self {
                        Self::Init => {
                            Self::from_index(format, reader, cx)
                        },
                        Self::Index(dec) => {
                            ::futures::ready!(dec.poll_decode(format, reader, cx))
                            .and_then(|idx| Self::from_data(&idx, format, reader, cx))
                        }
                        #(#transitions)*
                        Self::Fini => {
                            ::core::result::Result::Err(__F::invalid_input_err())
                        }
                    };
        
                    match res {
                        ::core::result::Result::Ok(status) => {
                            match status {
                                ::diny::backend::DecodeStatus::Ready(d) => {
                                    *self = Self::Fini;
                                    ::core::task::Poll::Ready(Ok(d))
                                }
                                ::diny::backend::DecodeStatus::Pending(p) => {
                                    *self = p;
                                    ::core::task::Poll::Pending
                                }
                            }
                        },
                        ::core::result::Result::Err(e) => {
                            *self = Self::Fini;
                            ::core::task::Poll::Ready(Err(e))
                        }
                    }
                }
            }
        }
    }

    let (
        decode_impl,
        decode_type,
        decode_init
    ) =
        if vs.len() > 0 {
            (
                non_empty_fields(vs),
                quote! { __Decoder<__F> },
                quote! { Self::Decoder::<__F>::Init }
            )
        } else {
            (
                quote! { compile_error!("Empty variant enums are not currently supported") },
                quote! { ::diny::backend::DecodeEmptyEnum::<__F, Self> },
                quote! { <::diny::backend::DecodeEmptyEnum::<__F, Self> as ::diny::backend::Decode>::init() }
            )
        };

    quote! {
        type __Data = #type_name;

        #decode_impl

        impl ::diny::backend::Decodable for __Data {
            type Decoder<__F>
            where
                __F: ::diny::backend::FormatDecode,
            = #decode_type;
        }
    
        impl ::diny::backend::AsyncDeserialize for __Data {
            type Future<'r, __F, __R>
            where
                __F: 'r + ::diny::backend::FormatDeserialize,
                __R: 'r + ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
            = ::diny::backend::DeserializeExact<'r, __F, __R, Self, Self::Decoder::<__F>>;
    
            fn deserialize<'r, __F, __R>(format: &'r __F, reader: &'r mut __R) -> Self::Future<'r, __F, __R>
            where
                __F: ::diny::backend::FormatDeserialize,
                __R: ::futures::io::AsyncRead + ::futures::io::AsyncBufRead + ::core::marker::Unpin,
            {
                ::diny::backend::DeserializeExact::new(format, reader, #decode_init)
            }
        }
    }
}
