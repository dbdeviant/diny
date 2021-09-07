use std::fmt::Display;
use proc_macro2::TokenStream;
use quote::ToTokens;

newtype!(pub Errors(Vec<syn::Error>));


impl Errors {
    pub fn new() -> Self {
        Errors(Vec::new())
    }

    pub fn add_syn_error(&mut self, err: syn::Error) {
        self.0.push(err);
    }

    pub fn add_spanned_error<T, M>(&mut self, tokens: T, msg: M)
    where
        T: ToTokens,
        M: Display,
    {
        self.add_syn_error(syn::Error::new_spanned(tokens.into_token_stream(), msg))
    }

    pub fn into_compile_errors(self) -> TokenStream {
        self.0.iter().map(syn::Error::to_compile_error).collect()
    }
}

impl From<Errors> for Result<(), Vec<syn::Error>> {
    fn from(e: Errors) -> Self {
        if !e.0.is_empty() {
            Err(e.0)
        } else {
            Ok(())
        }
    }
}

impl From<Errors> for TokenStream {
    fn from(e: Errors) -> Self {
        e.into_compile_errors()
    }
}

impl Default for Errors {
    fn default() -> Self {
        Self::new()
    }
}
    