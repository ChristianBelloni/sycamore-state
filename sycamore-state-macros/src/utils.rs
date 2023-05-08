use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, GenericParam, Generics, PathArguments, Type,
    Variant,
};

use crate::models::{FeatureModel, GetSetType};

pub fn extract_features_from_attrs(
    attrs: Vec<Attribute>,
    ident: Ident,
    features: &mut FeatureModel,
) {
    for attr in attrs.iter() {
        let attr = match &attr.meta {
            syn::Meta::Path(_) => continue,
            syn::Meta::List(data) => data,
            syn::Meta::NameValue(_) => continue,
        };
        if !attr.path.is_ident(&ident) {
            continue;
        }
        let tkns = attr.tokens.clone();

        for tkn in tkns.into_iter() {
            match tkn {
                proc_macro2::TokenTree::Ident(ident) => match ident.to_string().as_str() {
                    "debug" => features.debug = true,
                    "clone" => features.clone = true,
                    "eq" => features.eq = true,
                    "ord" => features.ord = true,
                    _ => {}
                },
                _ => continue,
            }
        }
    }
}

pub fn extract_attribute_from_field(attrs: &Vec<Attribute>, ident: Ident) -> Option<Ident> {
    for attr in attrs.iter() {
        match &attr.meta {
            syn::Meta::Path(attr) => {
                for seg in attr.segments.iter() {
                    if seg.ident.to_string() == ident.to_string() {
                        return Some(seg.ident.clone());
                    }
                }
            }
            syn::Meta::List(data) => {
                if data
                    .tokens
                    .clone()
                    .into_iter()
                    .find(|val| match val {
                        proc_macro2::TokenTree::Ident(ident) => ident.to_string() == "inner_state",
                        _ => false,
                    })
                    .is_none()
                {
                    continue;
                }
                for seg in data.path.segments.iter() {
                    if seg.ident.to_string() == ident.to_string() {
                        return Some(seg.ident.clone());
                    }
                }
            }
            syn::Meta::NameValue(_) => continue,
        };
    }
    None
}

#[allow(dead_code)]
pub fn extract_attribute_from_variant(field: &Variant, ident: Ident) -> Option<Ident> {
    for attr in field.attrs.iter() {
        let attr = match &attr.meta {
            syn::Meta::Path(data) => data,
            syn::Meta::List(_) => continue,
            syn::Meta::NameValue(_) => continue,
        };
        for seg in attr.segments.iter() {
            if seg.ident.to_string() == ident.to_string() {
                return Some(seg.ident.clone());
            }
        }
    }
    None
}

pub fn s_lifetime() -> syn::Lifetime {
    named_lifetime("stateful")
}

pub fn named_lifetime(name: &str) -> syn::Lifetime {
    syn::Lifetime {
        apostrophe: Span::call_site(),
        ident: format_ident!("{}", name),
    }
}

pub fn format_ref_ident(ident: &Ident) -> Ident {
    format_ident!("Ref{}", ident)
}
pub fn format_rc_ident(ident: &Ident) -> Ident {
    format_ident!("Rc{}", ident)
}

#[allow(dead_code)]
pub fn prepend_type(label: &str, mut ty: Type) -> Type {
    let tmp = match &mut ty {
        Type::Path(data) => Some(data),
        _ => None,
    };
    let tmp = tmp.unwrap();
    let last = tmp.path.segments.last_mut().unwrap();
    last.ident = format_ident!("{}{}", label, last.ident);
    ty
}

#[allow(dead_code)]
pub fn remove_generics(ty: &mut Type) {
    let data = match ty {
        Type::Path(data) => data,
        _ => panic!("not supported"),
    };
    let last_mut = data.path.segments.last_mut().unwrap();
    last_mut.arguments = PathArguments::None;
}

pub fn extract_lifetimes(generics: &Generics) -> Punctuated<syn::LifetimeParam, Comma> {
    generics
        .params
        .clone()
        .into_iter()
        .filter_map(|a| {
            if let GenericParam::Lifetime(data) = a {
                Some(data)
            } else {
                None
            }
        })
        .collect::<Punctuated<_, Comma>>()
}

pub fn make_derive_features(features: &FeatureModel) -> TokenStream {
    let mut macros = Punctuated::<Ident, Comma>::new();
    if features.clone {
        macros.push(format_ident!("Clone"));
    }
    if features.eq {
        macros.push(format_ident!("PartialEq"));
        macros.push(format_ident!("Eq"));
    }
    if features.debug {
        macros.push(format_ident!("Debug"))
    }
    if features.ord {
        macros.push(format_ident!("PartialOrd"));
        macros.push(format_ident!("Ord"));
    }

    let token_stream = if macros.len() != 0 {
        quote! {
            #[derive(#macros)]
        }
    } else {
        quote! {}
    };
    token_stream
}

pub fn make_ref_derive_features(features: &FeatureModel) -> TokenStream {
    let mut macros = Punctuated::<Ident, Comma>::new();
    macros.push(format_ident!("Copy"));
    macros.push(format_ident!("Clone"));

    if features.eq {
        macros.push(format_ident!("PartialEq"));
        macros.push(format_ident!("Eq"));
    }
    if features.debug {
        macros.push(format_ident!("Debug"))
    }
    if features.ord {
        macros.push(format_ident!("PartialOrd"));
        macros.push(format_ident!("Ord"));
    }

    let token_stream = if macros.len() != 0 {
        quote! {
            #[derive(#macros)]
        }
    } else {
        quote! {}
    };
    token_stream
}

pub(crate) fn is_collection<F: GetSetType>(field: &F) -> Option<Type> {
    if extract_attribute_from_field(&field.attrs(), format_ident!("collection")).is_some() {
        match field.ref_ty().clone() {
            Type::Array(data) => {
                let ty = *data.elem;
                return Some(ty);
            }
            Type::Path(data) => {
                let last = data.path.segments.last().unwrap();
                match last.arguments.clone() {
                    PathArguments::AngleBracketed(inner) => {
                        let inner = inner.args.first().unwrap().clone();
                        let inner = match inner {
                            syn::GenericArgument::Type(inner) => inner,
                            _ => return None,
                        };
                        return Some(inner);
                    }
                    _ => return None,
                }
            }
            _ => {}
        };
        None
    } else {
        None
    }
}

pub fn stateful_ident() -> Ident {
    format_ident!("state")
}

pub fn collection_ident() -> Ident {
    format_ident!("collection")
}
