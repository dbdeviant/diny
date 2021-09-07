use proc_macro2::TokenStream;
use quote::ToTokens;
use crate::data;

pub enum VariantType<'a> {
    Unit,
    TypeRef(&'a data::TypeRef<'a>),
}

impl VariantType<'_> {
    pub fn to_token_stream(&self) -> TokenStream {
        match self {
            Self::Unit => quote! { () },
            Self::TypeRef(type_ref) => type_ref.to_token_stream(),
        }
    }
}

pub struct EncodedVariantGen<'a> {
    pub variant: &'a data::Variant<'a>,
    pub ctor: syn::Ident,
    pub type_ref: VariantType<'a>,
    pub this_method: syn::Ident,
}

impl<'a> EncodedVariantGen<'a> {
    fn for_enum(variant: &'a data::Variant<'a>) -> Self {
        let ctor = format_ident!("V{}", *variant.index);
        let type_ref = variant.fields.first().map_or_else(|| VariantType::Unit, |v| VariantType::TypeRef(&v.type_ref));
        let this_method = format_ident!("v{}", variant.index.index);

        Self {
            variant,
            ctor,
            type_ref,
            this_method,
        }
    }
}

newtype!(pub EncodedVariantsGen<'a>(Vec<EncodedVariantGen<'a>>));

impl<'a> From<&'a data::Variants<'a>> for EncodedVariantsGen<'a> {
    fn from(fields: &'a data::Variants<'a>) -> Self {
        Self(
            fields
            .iter()
            .map(|variant| EncodedVariantGen::for_enum(variant))
            .collect::<Vec<_>>()
        )        
    }
}