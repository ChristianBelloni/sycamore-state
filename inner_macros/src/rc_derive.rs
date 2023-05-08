use crate::models::{FieldWrapper, GetSetType};
use crate::utils::format_rc_ident;
use crate::utils::is_collection;
use quote::{quote, ToTokens};
use syn::*;

impl FieldWrapper {
    pub fn to_rc_decl(&self) -> Box<dyn ToTokens> {
        let is_stateful = self.is_stateful();
        let is_collection = self.is_collection();

        match &self {
            FieldWrapper::Enum(data) => {
                let mut field = Self::inner_to_rc_decl(data, is_stateful, is_collection);
                field.attrs.clear();
                Box::new(field)
            }
            FieldWrapper::Struct(data) => {
                let mut variant = Self::inner_to_rc_decl(data, is_stateful, is_collection);
                variant.attrs.clear();
                Box::new(variant)
            }
        }
    }
    fn inner_to_rc_decl<F: GetSetType>(field: &F, is_stateful: bool, is_collection: bool) -> F {
        if is_stateful && is_collection {
            Self::rc_stateful_collection_decl(field)
        } else if is_stateful {
            Self::rc_stateful_decl(field)
        } else if is_collection {
            Self::rc_collection_decl(field)
        } else {
            Self::rc_bare_decl(field)
        }
    }
    fn rc_stateful_collection_decl<F: GetSetType>(field: &F) -> F {
        let mut inner_type = is_collection(field).expect("Type must be a collection");
        let mut new_field = field.clone();

        if let Type::Path(data) = &mut inner_type {
            let mut last = data.path.segments.last_mut().expect("failed to parse type");
            last.ident = format_rc_ident(&last.ident);
        }
        *new_field.mut_ty() =
            Type::Verbatim(quote! { ::sycamore_state::RcCollectionSignal<#inner_type>});

        new_field
    }
    fn rc_stateful_decl<F: GetSetType>(field: &F) -> F {
        let mut new_field = field.clone();
        let mut old_ty = field.ref_ty().clone();
        if let Type::Path(data) = &mut old_ty {
            let mut last = data.path.segments.last_mut().expect("failed to parse type");
            last.ident = format_rc_ident(&last.ident);
        }
        *new_field.mut_ty() = Type::Verbatim(quote! {::sycamore::prelude::RcSignal<#old_ty>});
        new_field
    }
    fn rc_collection_decl<F: GetSetType>(field: &F) -> F {
        let inner_type = is_collection(field).expect("Type must be a collection");
        let mut new_field = field.clone();

        *new_field.mut_ty() =
            Type::Verbatim(quote! { ::sycamore_state::RcCollectionSignal<#inner_type>});

        new_field
    }
    fn rc_bare_decl<F: GetSetType>(field: &F) -> F {
        let old_ty = &field.ref_ty();
        let mut new_field = field.clone();
        *new_field.mut_ty() = Type::Verbatim(quote! { ::sycamore::prelude::RcSignal<#old_ty>});
        new_field
    }
}

impl FieldWrapper {
    pub fn to_rc_ctor(&self) -> Box<dyn ToTokens> {
        let is_stateful = self.is_stateful();
        let is_collection = self.is_collection();
        match &self {
            FieldWrapper::Enum(data) => {
                let res = Self::inner_to_enum_rc_ctor(data, is_stateful, is_collection);
                Box::new(res)
            }
            FieldWrapper::Struct(data) => {
                let res = Self::inner_to_struct_rc_ctor(data, is_stateful, is_collection);
                Box::new(res)
            }
        }
    }

    fn inner_to_struct_rc_ctor<F: GetSetType>(
        field: &F,
        is_stateful: bool,
        is_collection: bool,
    ) -> Expr {
        if is_stateful && is_collection {
            Self::rc_struct_stateful_collection_ctor(field)
        } else if is_stateful {
            Self::rc_struct_stateful_ctor(field)
        } else if is_collection {
            Self::rc_struct_collection_ctor(field)
        } else {
            Self::rc_struct_bare_ctor(field)
        }
    }

    fn inner_to_enum_rc_ctor<F: GetSetType>(
        field: &F,
        is_stateful: bool,
        is_collection: bool,
    ) -> Expr {
        if is_stateful && is_collection {
            Self::rc_enum_stateful_collection_ctor(field)
        } else if is_stateful {
            Self::rc_enum_stateful_ctor(field)
        } else if is_collection {
            Self::rc_enum_collection_ctor(field)
        } else {
            Self::rc_enum_bare_ctor(field)
        }
    }

    fn rc_struct_stateful_collection_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        let old_ty = is_collection(field).unwrap();
        let inner_type = if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_rc_ident(&last.ident);
            quote! {
                #rc_ident::new(a)
            }
        } else {
            quote!()
        };

        Expr::Verbatim(quote! {
            ::sycamore_state::RcCollectionSignal::new(data.#ident.into_iter().map(|a| #inner_type))
        })
    }
    fn rc_struct_stateful_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        let old_ty = field.ref_ty().clone();
        if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_rc_ident(&last.ident);
            Expr::Verbatim(quote! {
                ::sycamore::prelude::create_rc_signal(#rc_ident::new(data.#ident))
            })
        } else {
            Expr::Verbatim(quote!())
        }
    }
    fn rc_struct_collection_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        Expr::Verbatim(quote! {
            ::sycamore_state::RcCollectionSignal::new(data.#ident)
        })
    }
    fn rc_struct_bare_ctor<F: GetSetType>(field: &F) -> Expr {
        let ident = field.ident();
        Expr::Verbatim(quote! { ::sycamore::prelude::create_rc_signal(data.#ident)})
    }

    fn rc_enum_stateful_collection_ctor<F: GetSetType>(field: &F) -> Expr {
        let old_ty = is_collection(field).unwrap();
        let inner_type = if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_rc_ident(&last.ident);
            quote! {
                #rc_ident::new(a)
            }
        } else {
            quote!()
        };

        Expr::Verbatim(quote! {
            ::sycamore_state::RcCollectionSignal::new(data.into_iter().map(|a| #inner_type))
        })
    }
    fn rc_enum_stateful_ctor<F: GetSetType>(field: &F) -> Expr {
        let old_ty = field.ref_ty().clone();
        if let Type::Path(data) = &old_ty {
            let last = data.path.segments.last().expect("failed to parse type");
            let rc_ident = format_rc_ident(&last.ident);
            Expr::Verbatim(quote! {
                ::sycamore::prelude::create_rc_signal(#rc_ident::new(data))
            })
        } else {
            Expr::Verbatim(quote!())
        }
    }
    fn rc_enum_collection_ctor<F: GetSetType>(_field: &F) -> Expr {
        Expr::Verbatim(quote! {
            ::sycamore_state::RcCollectionSignal::new(data)
        })
    }
    fn rc_enum_bare_ctor<F: GetSetType>(_field: &F) -> Expr {
        Expr::Verbatim(quote! { ::sycamore::prelude::create_rc_signal(data)})
    }
}
