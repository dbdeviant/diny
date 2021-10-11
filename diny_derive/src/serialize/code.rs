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
                ||  quote! { ::diny::backend::StartEncodeStatus::Fini },
                |n| quote! { Self::#n(format, writer, data, cx) },
            );

            quote! {
                fn #this_method<__W>(format: &__F, writer: &mut __W, data: &__Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartEncodeStatus<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    match <<#type_ref as ::diny::backend::Encodable>::Encoder::<__F> as ::diny::backend::Encode>::start_encode(format, writer, &data.#field_name, cx) {
                        ::diny::backend::StartEncodeStatus::Fini         => #next,
                        ::diny::backend::StartEncodeStatus::Pending(enc) => ::diny::backend::StartEncodeStatus::Pending(Self::#ctor(enc)),
                        ::diny::backend::StartEncodeStatus::Error(err)   => ::diny::backend::StartEncodeStatus::Error(err),
                    }
                }
            }
        });

        let init_transition = gen_encode_chain(quote! { Self::start_encode(format, writer, data, cx) });

        let transitions = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let field_name = &field.field.id.field_name();
            let poll = quote! { enc.poll_encode(format, writer, &data.#field_name, cx) };

            let transition = match &field.next_method {
                None => gen_encode_poll_fini(poll),
                Some(n) => gen_encode_poll_chain(poll, quote! { Self::#n(format, writer, data, cx) }),
            };

            quote! {
                Self::#ctor(enc) => #transition
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
        
                fn init(_data: &Self::Data) -> Self {
                    Self::Init
                }

                fn start_encode<__W>(format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartEncodeStatus<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    Self::after_init(format, writer, data, cx)
                }
        
                fn poll_encode<__W>(&mut self, format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::PollEncodeStatus<<__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    match self {
                        Self::Init => #init_transition,
                        #(#transitions)*
                        Self::Fini => ::diny::backend::PollEncodeStatus::Error(__F::invalid_input_err())
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
                __W: 'w + ::diny::io::AsyncWrite + ::core::marker::Unpin,
                = ::diny::backend::SerializeAll<'w, __F, __W, Self, Self::Encoder<__F>>;
    
            fn serialize<'w, __F, __W>(&'w self, format: &'w __F, writer: &'w mut __W) -> Self::Future<'w, __F, __W>
            where
                __F: ::diny::backend::FormatSerialize,
                __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
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
                ||  quote! { ::diny::backend::StartDecodeStatus::Fini(()) },
                |n| quote! { Self::#n(format, reader, data, cx) },
            );
            
            quote! {
                fn #this_method<__R>(format: &__F, reader: &mut __R, data: &mut __PartialData, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartDecodeStatus<(), Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    <<#type_ref as ::diny::backend::Decodable>::Decoder::<__F> as ::diny::backend::Decode>::start_decode(format, reader, cx)
                    .and_then(
                        |d| { data.#field_name = ::core::option::Option::Some(d); #next },
                        Self::#ctor,
                    )
                }
            }
        });

        let init_transition = gen_decode_chain(
            &quote! { state.cursor },
            &quote! { __DecodeCursor },
            quote! { __DecodeCursor::after_init(format, reader, &mut state.data, cx) },
        );

        let transitions = encoded_fields.iter().map(|field| {
            let ctor = &field.ctor;
            let field_name = &field.field.id.field_name();

            let next = &field.next_method.as_ref().map_or_else(
                ||  quote! { ::diny::backend::StartDecodeStatus::Fini(()) },
                |n| quote! { __DecodeCursor::#n(format, reader, &mut state.data, cx) },
            );

            let poll_chain = gen_decode_poll_chain(
                &quote! { state.cursor },
                &quote! { __DecodeCursor },
                quote! { dec.poll_decode(format, reader, cx) },
                quote! {
                    |d| {
                        state.data.#field_name = ::core::option::Option::Some(d);
                        #next
                    }                    
                }
            );

            quote! {
                __DecodeCursor::#ctor(dec) => {
                    #poll_chain
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
                fn new() -> Self {
                    Self {
                        #(#partial_field_defaults,)*
                    }
                }

                fn into_data(self) -> ::core::option::Option<__Data> {
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
                fn new() -> Self {
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
                fn new() -> Self {
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
        
                fn start_decode<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartDecodeStatus<Self::Data, Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    let mut data = __PartialData::new();
                    match __DecodeCursor::after_init(format, reader, &mut data, cx) {
                        ::diny::backend::StartDecodeStatus::Fini(()) =>
                            match data.into_data() {
                                ::core::option::Option::None => ::diny::backend::StartDecodeStatus::Error(__F::invalid_data_err()),
                                ::core::option::Option::Some(d) => ::diny::backend::StartDecodeStatus::Fini(d),
                            },
                        ::diny::backend::StartDecodeStatus::Pending(cursor) => ::diny::backend::StartDecodeStatus::Pending(Self { state: ::core::option::Option::Some(__DecodeState { data, cursor }) }),
                        ::diny::backend::StartDecodeStatus::Error(e) => ::diny::backend::StartDecodeStatus::Error(e),
                    }
                }


                fn poll_decode<__R>(&mut self, format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::PollDecodeStatus<Self::Data, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    if let Some(state) = &mut self.state {
                        match &mut state.cursor {
                            __DecodeCursor::Init => {
                                #init_transition
                            }
                            #(#transitions)*
                            __DecodeCursor::Fini => return ::diny::backend::PollDecodeStatus::Error(__F::invalid_input_err()),
                        }
                        .and_then(|()| match self.state.take().unwrap().data.into_data() {
                            ::core::option::Option::None => ::diny::backend::PollDecodeStatus::Error(__F::invalid_data_err()),
                            ::core::option::Option::Some(d) => ::diny::backend::PollDecodeStatus::Fini(d),
                        })
                    } else {
                        ::diny::backend::PollDecodeStatus::Error(__F::invalid_input_err())
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
                __R: 'r + ::diny::io::AsyncBufRead + ::core::marker::Unpin,
            = ::diny::backend::DeserializeExact<'r, __F, __R, Self, Self::Decoder::<__F>>;
    
            fn deserialize<'r, __F, __R>(format: &'r __F, reader: &'r mut __R) -> Self::Future<'r, __F, __R>
            where
                __F: ::diny::backend::FormatDeserialize,
                __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
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
            let ctor = &variant.ctor;
            let type_ref = &variant.type_ref.to_token_stream();
            let this_method = &variant.this_method;

            quote! {
                fn #this_method<__W>(format: &__F, writer: &mut __W, data: &#type_ref, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartEncodeStatus<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    match <<#type_ref as ::diny::backend::Encodable>::Encoder::<__F> as ::diny::backend::Encode>::start_encode(format, writer, &data, cx) {
                        ::diny::backend::StartEncodeStatus::Fini         => ::diny::backend::StartEncodeStatus::Fini,
                        ::diny::backend::StartEncodeStatus::Pending(enc) => ::diny::backend::StartEncodeStatus::Pending(Self::#ctor(enc)),
                        ::diny::backend::StartEncodeStatus::Error(err)   => ::diny::backend::StartEncodeStatus::Error(err),        
                    }
                }
            }
        });

        let init_transition = gen_encode_chain(quote! { Self::after_init(format, writer, data, cx) });
        let index_transition = gen_encode_poll_chain(quote! { enc.poll_encode(format, writer, &index, cx) }, quote! { Self::after_index(format, writer, data, cx) } );

        let transitions = encoded_variants.iter().map(|variant| {
            let encode_ctor = &variant.ctor;
            let data_ctor = &variant.variant.ctor;

            let poll = match &variant.type_ref {
                VariantType::Unit       => {
                    let poll_fini = gen_encode_poll_fini(quote! { enc.poll_encode(format, writer, &(), cx) });
                    quote! { __Data::#data_ctor{}  => #poll_fini }
                }
                VariantType::TypeRef(_) => {
                    let poll_fini = gen_encode_poll_fini(quote! { enc.poll_encode(format, writer, d, cx) });
                    quote! { __Data::#data_ctor(d) => #poll_fini }
                }                
            };

            quote! {
                Self::#encode_ctor(enc) => {
                    match data {
                        #poll,
                        _ => { debug_assert!(false); ::diny::backend::PollEncodeStatus::Error(__F::invalid_input_err()) },
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
        
                fn after_init<__W>(format: &__F, writer: &mut __W, data: &__Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartEncodeStatus<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    let index = Self::variant_index(data);
                    match <<::diny::backend::internal::VariantIdx as ::diny::backend::Encodable>::Encoder::<__F> as ::diny::backend::Encode>::start_encode(format, writer, &index, cx) {
                        ::diny::backend::StartEncodeStatus::Fini         => Self::after_index(format, writer, data, cx),
                        ::diny::backend::StartEncodeStatus::Pending(enc) => ::diny::backend::StartEncodeStatus::Pending(Self::Index(index, enc)),
                        ::diny::backend::StartEncodeStatus::Error(e)     => ::diny::backend::StartEncodeStatus::Error(e)            
                    }
                }
                
                fn after_index<__W>(format: &__F, writer: &mut __W, data: &__Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartEncodeStatus<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
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
        
                fn init(_data: &Self::Data) -> Self {
                    Self::Init
                }

                fn start_encode<__W>(format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartEncodeStatus<Self, <__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    Self::after_init(format, writer, data, cx)
                }
        
                fn poll_encode<__W>(&mut self, format: &__F, writer: &mut __W, data: &Self::Data, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::PollEncodeStatus<<__F as ::diny::backend::Format>::Error>
                where
                    __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
                {
                    // Contract: 'data' must not be modified between calls
                    match self {
                        Self::Init => #init_transition,
                        Self::Index(index, enc) => {
                            debug_assert_eq!(*index, Self::variant_index(data));
                            #index_transition
                        }
                        #(#transitions)*
                        Self::Fini => {
                            debug_assert!(false);
                            ::diny::backend::PollEncodeStatus::Error(__F::invalid_input_err())
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
                __W: 'w + ::diny::io::AsyncWrite + ::core::marker::Unpin,
                = ::diny::backend::SerializeAll<'w, __F, __W, Self, Self::Encoder<__F>>;
    
            fn serialize<'w, __F, __W>(&'w self, format: &'w __F, writer: &'w mut __W) -> Self::Future<'w, __F, __W>
            where
                __F: ::diny::backend::FormatSerialize,
                __W: ::diny::io::AsyncWrite + ::core::marker::Unpin,
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

            let status_ctor = match &variant.type_ref {
                VariantType::Unit => quote! { |_| ::diny::backend::StartDecodeStatus::Fini(__Data::#data_ctor{}) },
                VariantType::TypeRef(_) => quote! { |d| ::diny::backend::StartDecodeStatus::Fini(__Data::#data_ctor(d)) },
            };

            quote! {
                fn #this_method<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartDecodeStatus<__Data, Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    <<#type_ref as ::diny::backend::Decodable>::Decoder::<__F> as ::diny::backend::Decode>::start_decode(format, reader, cx)
                    .and_then(
                        #status_ctor,
                        Self::#decode_ctor
                    )
                }
            }
        });

        let init_transition = gen_decode_chain(
            &quote! { *self },
            &quote! { Self },
            quote! { Self::from_index(format, reader, cx) }
        );

        let index_transition = gen_decode_poll_chain(
            &quote! { *self },
            &quote! { Self },
            quote! { dec.poll_decode(format, reader, cx) },
            quote! { |idx| Self::after_index(idx, format, reader, cx) }
        );

        let transitions = encoded_variants.iter().map(|variant| {
            let decode_ctor = &variant.ctor;
            let data_ctor = &variant.variant.ctor;
            let ctor = match &variant.type_ref {
                VariantType::Unit => quote! { #data_ctor{} },
                VariantType::TypeRef(_) => quote! { #data_ctor(_d) },
            };

            let poll_fini = gen_decode_poll_fini(
                &quote! { *self },
                &quote! { Self },
                quote! { dec.poll_decode(format, reader, cx) },
                quote! { |_d| __Data::#ctor }
            );

            quote! {
                Self::#decode_ctor(dec) => {
                    #poll_fini
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
                fn from_index<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartDecodeStatus<__Data, Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    <<diny::backend::internal::VariantIdx as ::diny::backend::Decodable>::Decoder::<__F> as ::diny::backend::Decode>::start_decode(format, reader, cx)
                    .and_then(
                        |idx| Self::after_index(idx, format, reader, cx),
                        Self::Index
                    )
                }
        
                fn after_index<__R>(index: ::diny::backend::internal::VariantIdx, format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartDecodeStatus<__Data, Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    match *index {
                        #(#dispatch,)*
                        _ => ::diny::backend::StartDecodeStatus::Error(__F::invalid_input_err()),
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

                fn start_decode<__R>(format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::StartDecodeStatus<Self::Data, Self, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    Self::from_index(format, reader, cx)
                }


                fn poll_decode<__R>(&mut self, format: &__F, reader: &mut __R, cx: &mut ::core::task::Context<'_>) -> ::diny::backend::PollDecodeStatus<Self::Data, <__F as ::diny::backend::Format>::Error>
                where
                    __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
                {
                    match self {
                        Self::Init => {
                            #init_transition
                        },
                        Self::Index(dec) => {
                            #index_transition
                        }
                        #(#transitions)*
                        Self::Fini => {
                            ::diny::backend::PollDecodeStatus::Error(__F::invalid_input_err())
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
                __R: 'r + ::diny::io::AsyncBufRead + ::core::marker::Unpin,
            = ::diny::backend::DeserializeExact<'r, __F, __R, Self, Self::Decoder::<__F>>;
    
            fn deserialize<'r, __F, __R>(format: &'r __F, reader: &'r mut __R) -> Self::Future<'r, __F, __R>
            where
                __F: ::diny::backend::FormatDeserialize,
                __R: ::diny::io::AsyncBufRead + ::core::marker::Unpin,
            {
                ::diny::backend::DeserializeExact::new(format, reader, #decode_init)
            }
        }
    }
}


fn gen_encode_chain(start: TokenStream) -> TokenStream {
    quote! {
        match #start {
            ::diny::backend::StartEncodeStatus::Fini => {
                *self = Self::Fini;
                ::diny::backend::PollEncodeStatus::Fini
            }
            ::diny::backend::StartEncodeStatus::Pending(enc) => {
                *self = enc;
                ::diny::backend::PollEncodeStatus::Pending
            }
            ::diny::backend::StartEncodeStatus::Error(e) => {
                *self = Self::Fini;
                ::diny::backend::PollEncodeStatus::Error(e)
            }
        }
    }
}

fn gen_encode_poll_chain(poll: TokenStream, next: TokenStream) -> TokenStream {
    let next = gen_encode_chain(next);
    quote! {
        match #poll {
            ::diny::backend::PollEncodeStatus::Fini => #next,
            ::diny::backend::PollEncodeStatus::Pending => {
                ::diny::backend::PollEncodeStatus::Pending
            }
            ::diny::backend::PollEncodeStatus::Error(e) => {
                *self = Self::Fini;
                ::diny::backend::PollEncodeStatus::Error(e)
            }
        }
    }
}

fn gen_encode_poll_fini(poll: TokenStream) -> TokenStream {
    quote! {
        match #poll {
            ::diny::backend::PollEncodeStatus::Fini => {
                *self = Self::Fini;
                ::diny::backend::PollEncodeStatus::Fini
            }
            ::diny::backend::PollEncodeStatus::Pending => {
                ::diny::backend::PollEncodeStatus::Pending
            }
            ::diny::backend::PollEncodeStatus::Error(e) => {
                *self = Self::Fini;
                ::diny::backend::PollEncodeStatus::Error(e)
            }
        }
    }
}


fn gen_decode_chain(lhs: &TokenStream, rhs: &TokenStream, start: TokenStream) -> TokenStream {
    quote! {
        match #start {
            ::diny::backend::StartDecodeStatus::Fini(d) => {
                #lhs = #rhs::Fini;
                ::diny::backend::PollDecodeStatus::Fini(d)
            }
            ::diny::backend::StartDecodeStatus::Pending(enc) => {
                #lhs = enc;
                ::diny::backend::PollDecodeStatus::Pending
            }
            ::diny::backend::StartDecodeStatus::Error(e) => {
                #lhs = #rhs::Fini;
                ::diny::backend::PollDecodeStatus::Error(e)
            }
        }
    }
}

fn gen_decode_poll_chain(lhs: &TokenStream, rhs: &TokenStream, poll: TokenStream, cont: TokenStream) -> TokenStream {
    let decode_chain = gen_decode_chain(lhs, rhs, quote! { (#cont)(d) });
    quote! {
        match #poll {
            #[allow(clippy::redundant_closure_call)]
            ::diny::backend::PollDecodeStatus::Fini(d) => {
                #decode_chain
            }
            ::diny::backend::PollDecodeStatus::Pending => {
                ::diny::backend::PollDecodeStatus::Pending
            }
            ::diny::backend::PollDecodeStatus::Error(e) => {
                #lhs = #rhs::Fini;
                ::diny::backend::PollDecodeStatus::Error(e)
            }
        }
    }
}

fn gen_decode_poll_fini(lhs: &TokenStream, rhs: &TokenStream, poll: TokenStream, fin: TokenStream) -> TokenStream {
    quote! {
        match #poll {
            ::diny::backend::PollDecodeStatus::Fini(d) => {
                #lhs = #rhs::Fini;
                #[allow(clippy::redundant_closure_call)]
                ::diny::backend::PollDecodeStatus::Fini((#fin)(d))
            }
            ::diny::backend::PollDecodeStatus::Pending => {
                ::diny::backend::PollDecodeStatus::Pending
            }
            ::diny::backend::PollDecodeStatus::Error(e) => {
                #lhs = #rhs::Fini;
                ::diny::backend::PollDecodeStatus::Error(e)
            }
        }
    }
}
