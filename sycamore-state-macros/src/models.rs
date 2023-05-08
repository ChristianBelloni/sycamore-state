use std::marker::PhantomData;

use crate::utils::make_derive_features;
use crate::utils::{
    collection_ident, extract_lifetimes, format_rc_ident, format_ref_ident,
    make_ref_derive_features, named_lifetime, s_lifetime, stateful_ident,
};
use proc_macro2::*;
use quote::quote;
use quote::ToTokens;
use syn::token::{Enum, Struct};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Field, GenericParam, Generics, LifetimeParam,
    Variant,
};
use syn::{AngleBracketedGenericArguments, Type};
use syn::{Expr, GenericArgument};

pub(crate) trait GenericContainer {
    fn generics(&self) -> Generics;
    fn make_ref_generic(&self) -> Generics {
        let mut new = Punctuated::<GenericParam, Comma>::new();
        new.push(GenericParam::Lifetime(LifetimeParam::new(s_lifetime())));
        self.make_generic(new)
    }

    fn make_rc_generic(&self) -> Generics {
        self.make_generic(Default::default())
    }

    fn make_generic(&self, add: Punctuated<GenericParam, Comma>) -> Generics {
        let mut generics = self.generics();
        let old = generics.params;
        let mut new = Punctuated::<GenericParam, Comma>::new();
        new.extend(add);
        new.extend(old);
        generics.params = new;
        generics
    }
}

#[derive(Default, Debug, Clone)]
pub struct FeatureModel {
    pub debug: bool,
    pub clone: bool,
    pub eq: bool,
    pub ord: bool,
}

#[derive(Clone)]
pub enum FieldWrapper {
    Enum(Variant),
    Struct(Field),
}

impl ToTokens for FieldWrapper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldWrapper::Enum(data) => data.to_tokens(tokens),
            FieldWrapper::Struct(data) => data.to_tokens(tokens),
        }
    }
}

pub struct Deriver<T> {
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<FieldWrapper>,
    pub features: FeatureModel,
    _marker: PhantomData<T>,
}

impl Deriver<Enum> {
    pub fn new_enum(
        ident: Ident,
        generics: Generics,
        variants: impl IntoIterator<Item = Variant>,
        features: FeatureModel,
    ) -> Deriver<Enum> {
        let fields = variants
            .into_iter()
            .map(|a| FieldWrapper::Enum(a))
            .collect();
        Self {
            ident,
            generics,
            fields,
            features,
            _marker: Default::default(),
        }
    }

    pub fn derive_rc_decl(&self) -> TokenStream {
        let new_ident = format_rc_ident(&self.ident);
        let generics = self.make_rc_generic();
        let derives = make_derive_features(&self.features);
        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| a.to_rc_decl());
        fields.extend(mapped);
        quote! {
            #derives
            pub enum #new_ident #generics {
                #fields
            }
        }
    }

    pub fn derive_ref_decl(&self) -> TokenStream {
        let new_ident = format_ref_ident(&self.ident);
        let generics = self.make_ref_generic();
        let derives = make_ref_derive_features(&self.features);
        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| a.to_ref_decl());
        fields.extend(mapped);
        quote! {
            #derives
            pub enum #new_ident #generics {
                #fields
            }
        }
    }

    pub fn derive_rc_ctor(&self) -> TokenStream {
        let ident = &self.ident;
        let new_ident = format_rc_ident(&ident);
        let generics = self.make_rc_generic();
        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| {
            let variant_ident = a.ident();
            let arg = a.to_rc_ctor();
            quote! {
                #ident::#variant_ident(data) => Self::#variant_ident (#arg)
            }
        });

        fields.extend(mapped);

        quote! {
            impl #generics #new_ident #generics {
                pub fn new (data: #ident #generics) -> #new_ident # generics {
                    match data {
                        #fields
                    }
                }
            }
        }
    }

    pub fn derive_ref_ctor(&self) -> TokenStream {
        let ident = &self.ident;
        let new_ident = format_ref_ident(&ident);

        let generics = self.make_ref_generic();
        let mut added_generics = Punctuated::new();

        added_generics.push(GenericParam::Lifetime(LifetimeParam::new(named_lifetime(
            "ctor_ref",
        ))));

        let original_generics = self.generics.clone();
        let ctor_generics = self.make_generic(added_generics);
        let lifetimes = extract_lifetimes(&generics);
        let bounds = lifetimes
            .into_iter()
            .map(|a| {
                let ident = a.lifetime;
                Expr::Verbatim(quote! {#ident: 'ctor_ref})
            })
            .collect::<Punctuated<_, Comma>>();

        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| {
            let variant_ident = a.ident();
            let arg = a.to_ref_ctor();
            quote! {
                #ident::#variant_ident(data) => Self::#variant_ident (#arg)
            }
        });

        fields.extend(mapped);

        quote! {
            impl #generics #new_ident #generics {
                pub fn new<'ctor_ref>(cx: ::sycamore::prelude::Scope<'ctor_ref>, data: #ident #original_generics ) -> #new_ident #ctor_generics
                where
                    #bounds,
                    'ctor_ref: 'stateful,
                    {
                    match data {
                        #fields
                    }
                }
            }
        }
    }
}

impl Deriver<Struct> {
    pub fn new_struct(
        ident: Ident,
        generics: Generics,
        fields: impl IntoIterator<Item = Field>,
        features: FeatureModel,
    ) -> Self {
        let fields = fields
            .into_iter()
            .map(|a| FieldWrapper::Struct(a))
            .collect();
        Self {
            ident,
            generics,
            fields,
            features,
            _marker: Default::default(),
        }
    }

    pub fn derive_rc_decl(&self) -> TokenStream {
        let new_ident = format_rc_ident(&self.ident);
        let generics = self.make_rc_generic();
        let derives = make_derive_features(&self.features);
        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| a.to_rc_decl());
        fields.extend(mapped);
        quote! {
            #derives
            pub struct #new_ident #generics {
                #fields
            }
        }
    }

    pub fn derive_rc_ctor(&self) -> TokenStream {
        let ident = &self.ident;
        let new_ident = format_rc_ident(&ident);
        let generics = self.make_rc_generic();
        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| {
            let ident = a.ident();
            let arg = a.to_rc_ctor();
            quote! {
                #ident: #arg
            }
        });

        fields.extend(mapped);

        quote! {
            impl #generics #new_ident #generics {
                pub fn new (data: #ident #generics) -> #new_ident # generics {
                    #new_ident {
                        #fields
                    }
                }
            }
        }
    }

    pub fn derive_ref_decl(&self) -> TokenStream {
        let new_ident = format_ref_ident(&self.ident);
        let generics = self.make_ref_generic();
        let derives = make_ref_derive_features(&self.features);
        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| a.to_ref_decl());
        fields.extend(mapped);
        quote! {
            #derives
            pub struct #new_ident #generics {
                #fields
            }
        }
    }

    pub fn derive_ref_ctor(&self) -> TokenStream {
        let ident = &self.ident;
        let new_ident = format_ref_ident(&ident);

        let generics = self.make_ref_generic();
        let mut added_generics = Punctuated::new();

        added_generics.push(GenericParam::Lifetime(LifetimeParam::new(named_lifetime(
            "ctor_ref",
        ))));

        let original_generics = self.generics.clone();
        let ctor_generics = self.make_generic(added_generics);
        let lifetimes = extract_lifetimes(&generics);
        let bounds = lifetimes
            .into_iter()
            .map(|a| {
                let ident = a.lifetime;
                Expr::Verbatim(quote! {#ident: 'ctor_ref})
            })
            .collect::<Punctuated<_, Comma>>();

        let mut fields = Punctuated::<_, Comma>::new();
        let mapped = self.fields.iter().map(|a| {
            let ident = a.ident();
            let arg = a.to_ref_ctor();
            quote! {
                #ident: #arg
            }
        });

        fields.extend(mapped);

        quote! {
            impl #generics #new_ident #generics {
                pub fn new<'ctor_ref>(cx: ::sycamore::prelude::Scope<'ctor_ref>, data: #ident #original_generics ) -> #new_ident #ctor_generics
                where
                    #bounds
                    {
                    #new_ident {
                        #fields
                    }
                }
            }
        }
    }
}

impl<T> GenericContainer for Deriver<T> {
    fn generics(&self) -> Generics {
        self.generics.clone()
    }
}

pub(crate) trait GetSetType: Clone {
    fn ident(&self) -> &Ident;
    fn ref_ty(&self) -> &Type;
    fn mut_ty(&mut self) -> &mut Type;
    fn attrs(&self) -> Vec<Attribute>;
}

impl GetSetType for Field {
    fn ref_ty(&self) -> &Type {
        &self.ty
    }

    fn mut_ty(&mut self) -> &mut Type {
        &mut self.ty
    }

    fn attrs(&self) -> Vec<Attribute> {
        self.attrs.clone()
    }

    fn ident(&self) -> &Ident {
        &self.ident.as_ref().unwrap()
    }
}

impl GetSetType for Variant {
    fn ref_ty(&self) -> &Type {
        &self.fields.iter().next().unwrap().ty
    }

    fn mut_ty(&mut self) -> &mut Type {
        &mut self.fields.iter_mut().next().unwrap().ty
    }

    fn attrs(&self) -> Vec<Attribute> {
        self.attrs.clone()
    }

    fn ident(&self) -> &Ident {
        &self.ident
    }
}

pub fn insert_lifetime_into_generics(last: &mut syn::PathSegment, lifetime: syn::Lifetime) {
    let mut to_add = Punctuated::<_, Comma>::new();
    to_add.push(GenericArgument::Lifetime(lifetime));
    match &mut last.arguments {
        syn::PathArguments::None => {
            last.arguments = syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: Default::default(),
                lt_token: Default::default(),
                args: to_add,
                gt_token: Default::default(),
            });
        }
        syn::PathArguments::AngleBracketed(generics) => {
            let old = generics.args.clone();
            generics.args.clear();
            generics.args.extend(to_add);
            generics.args.extend(old);
        }
        syn::PathArguments::Parenthesized(_) => panic!("not supported"),
    }
}

impl GetSetType for FieldWrapper {
    fn ref_ty(&self) -> &Type {
        match self {
            FieldWrapper::Enum(data) => data.ref_ty(),
            FieldWrapper::Struct(data) => data.ref_ty(),
        }
    }

    fn mut_ty(&mut self) -> &mut Type {
        match self {
            FieldWrapper::Enum(data) => data.mut_ty(),
            FieldWrapper::Struct(data) => data.mut_ty(),
        }
    }

    fn attrs(&self) -> Vec<Attribute> {
        match self {
            FieldWrapper::Enum(data) => data.attrs(),
            FieldWrapper::Struct(data) => data.attrs(),
        }
    }

    fn ident(&self) -> &Ident {
        match self {
            FieldWrapper::Enum(data) => data.ident(),
            FieldWrapper::Struct(data) => data.ident(),
        }
    }
}

impl FieldWrapper {
    pub(crate) fn is_collection(&self) -> bool {
        self.has_attribute(collection_ident())
    }

    pub(crate) fn is_stateful(&self) -> bool {
        self.has_attribute(stateful_ident())
    }

    pub(crate) fn has_attribute(&self, ident: Ident) -> bool {
        self.attrs()
            .iter()
            .find(|a| match &a.meta {
                syn::Meta::Path(data) => {
                    if let Some(seg) = data.segments.last() {
                        seg.ident == ident
                    } else {
                        false
                    }
                }
                syn::Meta::List(data) => {
                    let data = &data.path;
                    if let Some(seg) = data.segments.last() {
                        seg.ident == ident
                    } else {
                        false
                    }
                }
                syn::Meta::NameValue(_) => false,
            })
            .is_some()
    }
}
