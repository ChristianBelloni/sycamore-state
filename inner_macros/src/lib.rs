mod models;
mod rc_derive;
mod ref_derive;
mod utils;

use models::{Deriver, FeatureModel};
use proc_macro2::*;
use quote::{format_ident, quote};
use syn::DeriveInput;
use utils::extract_features_from_attrs;

#[proc_macro_derive(State, attributes(state, collection))]
pub fn entry_point(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_stateful(input.into()).into()
}

fn derive_stateful(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse2(input).unwrap();
    let attrs = ast.attrs;
    let generics = ast.generics;
    let struct_ident = ast.ident;
    match ast.data {
        syn::Data::Struct(data) => derive_struct(data, struct_ident, generics, attrs),
        syn::Data::Enum(data) => derive_enum(data, struct_ident, generics, attrs),
        syn::Data::Union(_) => panic!("not implemented for unions"),
    }
}

fn derive_enum(
    data: syn::DataEnum,
    struct_ident: Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
) -> TokenStream {
    let variants = data.variants;
    let mut features = FeatureModel::default();
    extract_features_from_attrs(attrs.clone(), format_ident!("state"), &mut features);
    let deriver = Deriver::new_enum(struct_ident, generics, variants, features);
    let derived_rc_decl = deriver.derive_rc_decl();
    let derived_ref_decl = deriver.derive_ref_decl();
    let derived_rc_ctor = deriver.derive_rc_ctor();
    let derived_ref_ctor = deriver.derive_ref_ctor();
    quote! {
        #derived_rc_decl
        #derived_ref_decl
        #derived_rc_ctor
        #derived_ref_ctor
    }
}

fn derive_struct(
    data: syn::DataStruct,
    struct_ident: Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
) -> TokenStream {
    let fields = data.fields;
    let fields = match fields {
        syn::Fields::Named(named) => named,
        syn::Fields::Unnamed(_) => panic!("not implemented for unnamed fields"),
        syn::Fields::Unit => panic!("not implemented for unit structs"),
    };
    let fields = fields.named;
    let mut features = FeatureModel::default();
    extract_features_from_attrs(attrs.clone(), format_ident!("state"), &mut features);
    let deriver = Deriver::new_struct(struct_ident, generics, fields, features);

    let derived_rc_decl = deriver.derive_rc_decl();
    let derived_rc_ctor = deriver.derive_rc_ctor();
    let derived_ref_decl = deriver.derive_ref_decl();
    let derived_ref_ctor = deriver.derive_ref_ctor();

    quote! {
        #derived_ref_decl
        #derived_ref_ctor
        #derived_rc_decl
        #derived_rc_ctor
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn basic_test() {
        // let derived = derive_stateful(
        //     quote! {
        //         #[derive(Debug, Clone)]
        //         pub struct MyState<'a> {
        //             pub field1: i32,
        //             pub lif: &'a str,
        //             #[stateful]
        //             pub field2: InnerState
        //         }
        //     }
        //     .into(),
        // );
        // println!("{:?}", derived.to_string());
        let _derived = derive_stateful(
            quote! {
                #[derive(Debug, Clone)]
                pub struct MyState<'a, T> {
                    pub field: i32,
                    pub ref_field: &'a str,
                    pub generic_field: T,
                    #[stateful]
                    pub field2: InnerState,
                    #[stateful]
                    pub field3: InnerState,
                    #[stateful]
                    pub inner_ref: InnerRefState<'a>,
                    #[collection]
                    #[stateful]
                    pub collection: Vec<String>
                }
            }
            .into(),
        );
    }
}
