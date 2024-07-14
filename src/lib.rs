// Date:   Tue Jul 09 14:18:05 2024
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian

use std::collections::BTreeMap;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Fields, FieldsUnnamed, ItemEnum, Variant};
use quote::quote;

mod attr;
use attr::AutoFromAttributes;

fn path_to_string(path: &syn::TypePath) -> String {
    if path.qself.is_some() {
        panic!("qself is not allowed");
    }
    let path = &path.path;
    let segments = path.segments.iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    if path.leading_colon.is_some() {
        format!("::{}", segments)
    } else {
        segments
    }
}

#[proc_macro_attribute]
pub fn auto_throw(attrs: TokenStream, input: TokenStream) -> TokenStream {
    // println!("attr: \"{attr}\"");
    let AutoFromAttributes { disabled, .. } = 
        parse_macro_input!(attrs as AutoFromAttributes);

    let og = parse_macro_input!(input as ItemEnum);
    let ItemEnum { variants, ident: enum_name, .. } = og.clone();

    // check fields types
    // only unnamed fields with length of 1 is allowed
    let mut type2ident = BTreeMap::<String, String>::new();
    let variants = variants.into_iter()
        .filter(|var| {
            if disabled.contains(&var.ident) {
                return false;
            }

            match &var.fields {
                Fields::Unnamed(unnamed) => {
                    let unnamed = &unnamed.unnamed;
                    if unnamed.len() != 1 {
                        return false
                    }
                    let first = unnamed.first().unwrap();
                    match &first.ty {
                        syn::Type::Path(type_path) => {
                            let type_string = path_to_string(type_path);
                            let for_help = type_string.clone();
                            if let Some(old) = type2ident.insert(type_string, var.ident.to_string()) {
                                panic!("Variant {} and {} has the same type {}", 
                                    old, var.ident.to_string(), for_help);
                            }
                            true
                        },
                        _ => false
                    }
                },
                _ => false
            }
        })
        .collect::<Vec<_>>();

    let impls = variants.into_iter()
        .map(|var| {
            let Variant { fields, ident, .. } = var;
            let fields = match fields {
                Fields::Named(_) => panic!("Named field {} is not allowed", ident.to_string()),
                Fields::Unnamed(FieldsUnnamed {unnamed, ..}) => quote!(#unnamed),
                v => return quote!(#v)
            };
            quote! {
                impl From<#fields> for #enum_name {
                    fn from(item: #fields) -> Self {
                        Self::#ident(item)
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    TokenStream::from(quote! {
        #og
        #(#impls)*
    })
}
