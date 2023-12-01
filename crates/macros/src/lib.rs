//! ### Implement the `Partial<T>` type similar to Typescript language in rust.
//!
//! Constructs a type with all properties of Type set to optional. This utility will return a type that represents all subsets of a given type.
//!
//! # Example
//!
//! ```
//! 
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DataStruct, DeriveInput};

///
/// # Examples
/// 
/// ```
/// 
/// ```
#[proc_macro_derive(Partial, attributes(partial))]
pub fn partial_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = input.vis;
    let generics = input.generics;
    let name = input.ident.clone();
    let where_clause = generics.where_clause.clone();
    let params = input
        .attrs
        .iter()
        .find(|attr| is_partial_attr(*attr))
        .map(|attr| parse_attrs(attr))
        .unwrap();

    let attrs = input.attrs.iter().filter(|attr| !is_partial_attr(*attr));
    let items = require_struct(input.data.clone())
        .fields
        .into_iter()
        .map(|field| {
            let attrs = field.attrs.iter().filter(|attr| !is_partial_attr(*attr));
            let key = field.ident;
            let vis = field.vis;
            let ty = field.ty;
            let ty = field
                .attrs
                .iter()
                .filter(|attr| is_partial_attr(*attr))
                .next()
                .map(|attr| {
                    parse_attrs(attr)
                        .from
                        .map(|name| {
                            let partial_name = Ident::new(&name, input.ident.span().clone());
                            quote! { #partial_name }
                        })
                        .unwrap()
                })
                .unwrap_or_else(|| quote! { #ty });

            quote! {
                #(#attrs)*
                #vis #key: Option<#ty>,
            }
        });

    let alias = Ident::new(&params.alias, input.ident.span().clone());
    let derives = params
        .derives
        .into_iter()
        .map(|name| Ident::new(&name, input.ident.span().clone()))
        .map(|name| quote! { #[derive(#name)] });

    let setters = require_struct(input.data).fields.into_iter().map(|field| {
        let key = field.ident;
        if field
            .attrs
            .iter()
            .filter(|attr| is_partial_attr(*attr))
            .next()
            .is_some()
        {
            quote! {
                if let Some(v) = partial.#key {
                    self.#key.from_partial(v);
                }
            }
        } else {
            quote! {
                if let Some(v) = partial.#key {
                    self.#key = v;
                }
            }
        }
    });

    TokenStream::from(quote! {
        #(#derives)*
        #(#attrs)*
        #vis struct #alias #generics #where_clause {
            #(#items)*
        }

        impl #generics #name #generics #where_clause {
            #[inline]
            #vis fn from_partial(&mut self, partial: #alias #generics) {
                #(#setters)*
            }
        }
    })
}

fn require_struct(data: Data) -> DataStruct {
    if let Data::Struct(structs) = data {
        structs
    } else {
        panic!("only supports struct.");
    }
}

fn is_partial_attr(attr: &Attribute) -> bool {
    if let Some(path) = attr
        .meta
        .require_list()
        .ok()
        .map(|list| Some(list.path.clone()))
        .unwrap_or_else(|| attr.meta.require_path_only().ok().cloned())
    {
        if let Some(segment) = path.segments.first() {
            return segment.ident.to_string().as_str() == "partial";
        }
    }

    false
}

#[derive(Default, Debug)]
struct AttrParams {
    alias: String,
    derives: Vec<String>,
    from: Option<String>,
}

fn parse_attrs(attr: &Attribute) -> AttrParams {
    let mut params = AttrParams::default();

    attr.meta
        .require_list()
        .unwrap()
        .clone()
        .tokens
        .to_string()
        .split(',')
        .map(|kv| kv.trim())
        .for_each(|kv| {
            let mut kv = kv.split('=').map(|item| item.trim()).take(2);
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                match key {
                    "alias" => {
                        params.alias = value.to_string().replace('"', "");
                    }
                    "from" => {
                        params.from = Some(value.to_string().replace('"', ""));
                    }
                    "derives" => {
                        params.derives = value
                            .replace('[', "")
                            .replace(']', "")
                            .split('+')
                            .map(|item| item.trim().to_string())
                            .collect::<Vec<String>>();
                    }
                    _ => (),
                }
            }
        });

    params
}
