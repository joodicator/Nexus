use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated, parse::{Parse, ParseStream}, token::{Lt, Gt, Comma},
};


mod static_impl_generics;
pub use static_impl_generics::static_impl_generics;


pub struct AngleBracketList<T> {
    pub lt_token: Option<Lt>,
    pub items: Punctuated<T, Comma>,
    pub gt_token: Option<Gt>,
}

impl<T: Parse> Parse for AngleBracketList<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut list = AngleBracketList{
            lt_token: None,
            items: Punctuated::new(),
            gt_token: None,
        };
        list.lt_token = input.parse()?;
        if list.lt_token.is_none() { return Ok(list); }
        while !input.lookahead1().peek(Gt) {
            list.items.push_value(input.parse()?);
            match input.parse()? {
                Some(comma) => list.items.push_punct(comma),
                None        => break,
            }
        }
        list.gt_token = Some(input.parse()?);
        Ok(list)
    }
}

impl<T: ToTokens> ToTokens for AngleBracketList<T> {
    fn to_tokens(&self, output: &mut TokenStream) {
        self.lt_token.to_tokens(output);
        self.items.to_tokens(output);
        self.gt_token.to_tokens(output);
    }
}
