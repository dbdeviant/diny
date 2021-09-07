use crate::data;

pub struct EncodedFieldGen<'a> {
    pub field: &'a data::Field<'a>,
    pub ctor: syn::Ident,
    pub this_method: syn::Ident,
    pub next_method: Option<syn::Ident>,
}

impl<'a> EncodedFieldGen<'a> {
    fn for_struct(field: &'a data::Field<'a>, is_last: bool) -> Self {
        let ctor = format_ident!("F{}", *field.id.index);

        let this_method =
            if field.id.index.index == 0 {
                format_ident!("after_init")
            } else {
                format_ident!("after_f{}", field.id.index.index - 1)
            };

        let next_method =
            if is_last {
                None
            } else {
                Some(format_ident!("after_f{}", *field.id.index))
            };

        Self {
            field,
            ctor,
            this_method,
            next_method,
        }
    }
}

newtype!(pub EncodedFieldsGen<'a>(Vec<EncodedFieldGen<'a>>));

impl<'a> From<&'a data::Fields<'a>> for EncodedFieldsGen<'a> {
    fn from(fields: &'a data::Fields<'a>) -> Self {
        let n = fields.len() as u32;
        Self(
            fields
            .iter()
            .map(|field| EncodedFieldGen::for_struct(field, n-1 == field.id.index.index))
            .collect::<Vec<_>>()
        )        
    }
}