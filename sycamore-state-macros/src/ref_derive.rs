use crate::models::{insert_lifetime_into_generics, FieldWrapper, GetSetType};
use crate::utils::{format_ref_ident, is_collection, s_lifetime};
use quote::{quote, ToTokens};
use syn::*;

impl FieldWrapper {
    pub fn to_ref_decl(&self) -> Box<dyn ToTokens> {
        let is_stateful = self.is_stateful();
        let is_collection = self.is_collection();

        match &self {
            FieldWrapper::Enum(data) => {
                let mut field = Self::inner_to_ref_decl(data, is_stateful, is_collection);
                field.attrs.clear();
                Box::new(field)
            }
            FieldWrapper::Struct(data) => {
                let mut variant = Self::inner_to_ref_decl(data, is_stateful, is_collection);
                variant.attrs.clear();
                Box::new(variant)
            }
        }
    }
    fn inner_to_ref_decl<F: GetSetType>(field: &F, is_stateful: bool, is_collection: bool) -> F {
        if is_stateful && is_collection {
            Self::to_ref_stateful_collection_decl(field)
        } else if is_stateful {
            Self::to_ref_stateful_decl(field)
        } else if is_collection {
            Self::to_ref_collection_decl(field)
        } else {
            Self::to_ref_bare_decl(field)
        }
    }
    fn to_ref_stateful_collection_decl<F: GetSetType>(field: &F) -> F {
        let mut inner_type = is_collection(field).expect("Type must be a collection");
        let mut new_field = field.clone();

        if let Type::Path(data) = &mut inner_type {
            let mut last = data.path.segments.last_mut().expect("failed to parse type");
            insert_lifetime_into_generics(last, s_lifetime());
            last.ident = format_ref_ident(&last.ident);
        }
        *new_field.mut_ty() =
            Type::Verbatim(quote! { ::sycamore_state::RefCollectionSignal<'stateful, #inner_type>});

        new_field
    }
    fn to_ref_stateful_decl<F: GetSetType>(field: &F) -> F {
        let mut new_field = field.clone();
        let mut old_ty = field.ref_ty().clone();
        if let Type::Path(data) = &mut old_ty {
            let mut last = data.path.segments.last_mut().expect("failed to parse type");
            last.ident = format_ref_ident(&last.ident);
            insert_lifetime_into_generics(last, s_lifetime());
        }
        *new_field.mut_ty() =
            Type::Verbatim(quote! {&'stateful ::sycamore::prelude::Signal<#old_ty>});
        new_field
    }
    fn to_ref_collection_decl<F: GetSetType>(field: &F) -> F {
        let inner_type = is_collection(field).expect("Type must be a collection");
        let mut new_field = field.clone();

        *new_field.mut_ty() =
            Type::Verbatim(quote! { ::sycamore_state::RefCollectionSignal<'stateful, #inner_type>});

        new_field
    }
    fn to_ref_bare_decl<F: GetSetType>(field: &F) -> F {
        let old_ty = &field.ref_ty();
        let mut new_field = field.clone();
        *new_field.mut_ty() =
            Type::Verbatim(quote! { &'stateful ::sycamore::prelude::Signal<#old_ty>});
        new_field
    }
}

impl FieldWrapper {
    pub fn to_ref_ctor(&self) -> Box<dyn ToTokens> {
        let is_stateful = self.is_stateful();
        let is_collection = self.is_collection();
        match &self {
            FieldWrapper::Enum(data) => {
                let res = Self::inner_to_enum_ref_ctor(data, is_stateful, is_collection);
                Box::new(res)
            }
            FieldWrapper::Struct(data) => {
                let res = Self::inner_to_struct_ref_ctor(data, is_stateful, is_collection);
                Box::new(res)
            }
        }
    }
    fn inner_to_struct_ref_ctor<F: GetSetType>(
        field: &F,
        is_stateful: bool,
        is_collection: bool,
    ) -> Expr {
        if is_stateful && is_collection {
            Self::to_ref_struct_stateful_collection_ctor(field)
        } else if is_stateful {
            Self::to_ref_struct_stateful_ctor(field)
        } else if is_collection {
            Self::to_ref_struct_collection_ctor(field)
        } else {
            Self::to_ref_struct_bare_ctor(field)
        }
    }
    fn to_ref_struct_stateful_collection_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        let old_ty = is_collection(field).expect("failed to parse collection");
        let inner_type = if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_ref_ident(&last.ident);
            quote! {
                #rc_ident::new(cx, a)
            }
        } else {
            quote!()
        };

        Expr::Verbatim(quote! {
            unsafe{ ::sycamore_state::RefCollectionSignal::new(cx, data.#ident.into_iter().map(|a| #inner_type).collect::<Vec<_>>())}
        })
    }
    fn to_ref_struct_stateful_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        let old_ty = field.ref_ty().clone();
        if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_ref_ident(&last.ident);
            Expr::Verbatim(quote! {
                unsafe{::sycamore::reactive::create_signal_unsafe(cx, #rc_ident::new(cx, data.#ident))}
            })
        } else {
            Expr::Verbatim(quote!())
        }
    }
    fn to_ref_struct_collection_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        Expr::Verbatim(quote! {
            unsafe{ ::sycamore_state::RefCollectionSignal::new(cx, data.#ident)}
        })
    }
    fn to_ref_struct_bare_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        Expr::Verbatim(
            quote! { unsafe{::sycamore::reactive::create_signal_unsafe(cx, data.#ident)}},
        )
    }

    fn inner_to_enum_ref_ctor<F: GetSetType>(
        field: &F,
        is_stateful: bool,
        is_collection: bool,
    ) -> Expr {
        if is_stateful && is_collection {
            Self::to_ref_enum_stateful_collection_ctor(field)
        } else if is_stateful {
            Self::to_ref_enum_stateful_ctor(field)
        } else if is_collection {
            Self::to_ref_enum_collection_ctor(field)
        } else {
            Self::to_ref_enum_bare_ctor(field)
        }
    }
    fn to_ref_enum_stateful_collection_ctor<F: GetSetType>(field: &F) -> Expr {
        let old_ty = is_collection(field).expect("failed to parse collection");
        let inner_type = if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_ref_ident(&last.ident);
            quote! {
                #rc_ident::new(cx, a)
            }
        } else {
            quote!()
        };

        Expr::Verbatim(quote! {
            unsafe{ ::sycamore_state::RefCollectionSignal::new(cx, data.into_iter().map(|a| #inner_type).collect::<Vec<_>>())}
        })
    }
    fn to_ref_enum_stateful_ctor<F: GetSetType>(field: &F) -> Expr {
        let old_ty = field.ref_ty().clone();
        if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_ref_ident(&last.ident);
            Expr::Verbatim(quote! {
                unsafe{::sycamore::reactive::create_signal_unsafe(cx, #rc_ident::new(cx, data))}
            })
        } else {
            Expr::Verbatim(quote!())
        }
    }
    fn to_ref_enum_collection_ctor<F: GetSetType>(_field: &F) -> Expr {
        Expr::Verbatim(quote! {
            unsafe{ ::sycamore_state::RefCollectionSignal::new(cx, data)}
        })
    }
    fn to_ref_enum_bare_ctor<F: GetSetType>(_field: &F) -> Expr {
        Expr::Verbatim(quote! { unsafe{::sycamore::reactive::create_signal_unsafe(cx, data)}})
    }
}
