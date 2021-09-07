pub mod errors;

pub use errors::Errors;

pub type Result<T> = core::result::Result<T, ()>;

newtype!(pub Input<'a>(&syn::DeriveInput));

pub struct Def<'a> {
    pub name: TypeName<'a>,
    pub shape: Shape<'a>,
    pub input: Input<'a>,
}

newtype!(pub TypeName<'a>(&syn::Ident));
newtype!(pub TypeRef<'a>(&syn::Type));

pub enum Shape<'a> {
    Enum(Variants<'a>),
    Struct(Fields<'a>),
}

newtype!(pub Variants<'a>(Vec<Variant<'a>>));
newtype!(pub Fields<'a>(Vec<Field<'a>>));

pub enum FieldsStyle {
    Named,
    Unnamed,
}

pub struct Variant<'a> {
    pub ctor: VariantCtor<'a>,
    pub index: VariantIndex,
    pub fields: Fields<'a>,
}


newtype!(pub VariantCtor<'a>(&syn::Ident));
newtype!(pub VariantIndex(syn::Index));

pub struct Field<'a> {
    pub id: FieldId<'a>,
    pub type_ref: TypeRef<'a>,
}

pub struct FieldId<'a> {
    pub index: FieldIndex,
    pub name: Option<FieldName<'a>>,
}

newtype!(pub FieldIndex(syn::Index));
newtype!(pub FieldName<'a>(&syn::Ident));


impl<'a> Def<'a> {
    pub fn parse_input(input: &'a syn::DeriveInput, errors: &mut Errors) -> Result<Self> {
        let shape = Shape::parse_input(&input.data, input, errors)?;

        Ok(Self {
            name: (&input.ident).into(),
            shape,
            input: input.into(),
        })
    }
}

impl<'a> Shape<'a> {
    pub fn parse_input(data: &'a syn::Data, input: &'a syn::DeriveInput, errors: &mut Errors) -> Result<Self> {
        let shape = match data {
            syn::Data::Struct(s) => {
                match &s.fields {
                    syn::Fields::Named(fields)   => Shape::Struct(Fields::parse_input(fields.named.iter(), errors)?),
                    syn::Fields::Unnamed(fields) => Shape::Struct(Fields::parse_input(fields.unnamed.iter(), errors)?),
                    syn::Fields::Unit            => Shape::Struct(Fields::unit()),
                }
            },
            syn::Data::Enum(e) => Shape::Enum(Variants::parse_input(e.variants.iter().enumerate(), errors)?),
            syn::Data::Union(_) => {
                errors.add_spanned_error(input, "Diny does not support derivation of unions");
                return Err(());
            }
        };

        Ok(shape)
    }
}

impl<'a> Fields<'a> {
    pub fn unit() -> Self {
        Fields(Vec::new())
    }

    pub fn style(&self) -> FieldsStyle {
        if !self.0.is_empty() && self.0[0].id.name.is_some() {
            FieldsStyle::Named
        } else {
            FieldsStyle::Unnamed
        }
    }

    pub fn parse_input<Fs>(fields: Fs, errors: &mut Errors) -> Result<Self>
    where
        Fs: Iterator<Item = &'a syn::Field>,
    {
        enum AllSame {
            Unknown,
            Named,
            Unnamed,
        }

        fields.enumerate().scan(AllSame::Unknown, |all_same, (idx, field)| {
            Some(match all_same {
                AllSame::Unknown => {
                    *all_same =
                        if field.ident.is_some() {
                            AllSame::Named
                        } else {
                            AllSame::Unnamed
                        };

                    Ok(Field::parse_input(idx, field))
                }
                AllSame::Named => {
                    if field.ident.is_some() {
                        Ok(Field::parse_input(idx, field))
                    } else {
                        errors.add_spanned_error(&field.ident, "Named field found within unnamed field definition");
                        Err(())
                    }
                }
                AllSame::Unnamed => {
                    if field.ident.is_none() {
                        Ok(Field::parse_input(idx, field))
                    } else {
                        errors.add_spanned_error(&field.ident, "Unnamed field found within named field definition");
                        Err(())
                    }
                }
            })
        })
        .collect::<Result<Vec<Field>>>()
        .map(|fs| fs.into())
    }
}

impl<'a> Field<'a> {
    pub fn parse_input(idx: usize, field: &'a syn::Field) -> Self {
        Field {
            id: FieldId::new(idx, &field.ident),
            type_ref: (&field.ty).into(),
        }
    }
}

impl<'a> FieldId<'a> {
    pub fn new(idx: usize, ident: &'a Option<syn::Ident>) -> Self {
        Self {
            index: idx.into(),
            name: ident.as_ref().map(|i| i.into()),
        }
    }
}

impl FieldId<'_> {
    pub fn field_name(&self) -> proc_macro2::TokenStream {
        use quote::ToTokens;
        match &self.name {
            None => self.index.to_token_stream(),
            Some(n) => n.to_token_stream(),
        }
    }

    #[allow(dead_code)]
    pub fn var_name(&self) -> proc_macro2::TokenStream {
        use quote::ToTokens;
        match &self.name {
            None => format_ident!("_{}", self.index.0).to_token_stream(),
            Some(n) => n.to_token_stream(),
        }
    }
}

impl FieldIndex {
    pub fn new(idx: usize) -> Self {
        FieldIndex(idx.into())
    }
}

impl From<usize> for FieldIndex {
    fn from(idx: usize) -> Self {
        Self::new(idx)
    }
}


impl<'a> Variants<'a> {
    pub fn parse_input<Vs>(variants: Vs, errors: &mut Errors) -> Result<Self> 
    where
        Vs: Iterator<Item = (usize, &'a syn::Variant)>,
    {
        variants
            .map(|v| Variant::parse_input(v.0, v.1, errors))
            .collect::<Result<Vec<_>>>()
            .map(|vs| vs.into())
    }
}

impl<'a> Variant<'a> {
    pub fn parse_input(idx: usize, variant: &'a syn::Variant, errors: &mut Errors) -> Result<Self> {
        Ok(Variant {
            ctor: (&variant.ident).into(),
            index: VariantIndex(idx.into()),
            fields: Fields::parse_input(variant.fields.iter(), errors)?,
        })
    }
}


impl quote::ToTokens for TypeName<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for TypeRef<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for VariantCtor<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for VariantIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for FieldName<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for FieldIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}
