// Date:   Sun Jul 14 17:31:56 2024
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian

use std::collections::HashMap;

use syn::{
    parse::{Parse, ParseStream}, punctuated::Punctuated, Ident, Token
};

const EXPECTED_ATTRS: &[&'static str] = &["disabled"];

pub struct Attribute {
    pub attr_name: Ident,
    pub idents: Vec<Ident>
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr_name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let idents;
        let _ = syn::bracketed!(idents in input);
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(&idents)?;

        Ok(Attribute {
            attr_name,
            idents: idents.into_iter().collect::<Vec<_>>()
        })
    }
}

pub struct AutoFromAttributes {
    pub disabled: Vec<Ident>
}

impl Parse for AutoFromAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Punctuated::<Attribute, Token![,]>::parse_terminated(input)?;

        let mut map = HashMap::<String, Attribute>::new();

        // Filter unexpected attributes
        for attr in attrs.into_iter() {
            if !EXPECTED_ATTRS.contains(&attr.attr_name.to_string().as_str()) {
                return Err(syn::Error::new(attr.attr_name.span(),
                    &format!("Unexpected attribute {}", attr.attr_name)));
            }

            // only for error message
            let attr_span = attr.attr_name.span();
            let attr_name_string = attr.attr_name.to_string();
            if let Some(_) = map.insert(attr.attr_name.to_string(), attr) {
                return Err(syn::Error::new(attr_span,
                    &format!("Attribute {} appeared multiple times", attr_name_string)));
            }
        }

        let mut get_attr = |name: &str| {
            map.remove(name)
                .map(|attr| attr.idents)
                .or(Some(Vec::new()))
                .unwrap()
        };

        Ok(AutoFromAttributes {
            disabled: get_attr("disabled")
        })
    }
}

