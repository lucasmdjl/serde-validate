/*
 * serde-validate-macro - A procedural macro that validates the deserialization of a struct
 *
 * Copyright (C) 2024 Lucas M. de Jong Larrarte
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! # serde-validate-macro
//!
//! This crate provides the `validate_deser` procedural serde-validate-macro for the `serde-validate` crate.
//!
//! Users should prefer using the `serde-validate` crate.


extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Index, Variant};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;


/// Attribute serde-validate-macro to derive deserialization with validation for a struct or enum.
///
/// This serde-validate-macro generates a helper struct to deserialize the original struct or enum and
/// then validates the deserialized data using the `serde_validate::Validate` trait. If validation fails,
/// a deserialization error is returned.
#[proc_macro_attribute]
pub fn validate_deser(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let helper_name = Ident::new(&format!("__ValidDeserialize{name}"), name.span());

    let HelperData { helper_def, init_from_helper } = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => build_named_struct(&name, &helper_name, &fields.named),
            Fields::Unnamed(ref fields) => build_unnamed_struct(&name, &helper_name, &fields.unnamed),
            Fields::Unit => build_unit_struct(&name, &helper_name),
        }
        Data::Enum(ref data) => build_enum(&name, &helper_name, &data.variants),
        Data::Union(_) => {unimplemented!()}
    };

    let tokens = quote! {
        #input

        #[derive(serde::Deserialize)]
        #helper_def

        impl <'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let helper = #helper_name::deserialize(deserializer)?;
                let instance = #init_from_helper;
                instance.validated().map_err(serde::de::Error::custom)
            }
        }

    };

    tokens.into()
}


struct HelperData {
    helper_def: proc_macro2::TokenStream,
    init_from_helper: proc_macro2::TokenStream,
}

fn build_named_struct(name: &Ident, helper_name: &Ident, fields: &Punctuated<Field, Comma>) -> HelperData {
    let helper_def = named_def(helper_name, fields);

    let helper_def = quote! {
        struct #helper_def
    };

    let init_from_helper = init_from_named(name, fields);

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn named_def(name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        quote! { #name: #ty}
    });
    quote! {
        #name {
            #( #fields),*
        }
    }
}

fn init_from_named(name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let init_fields = fields.iter().map(|field| {
        let name = &field.ident;
        quote! { #name: helper.#name }
    });

    quote! {
        #name {
            #( #init_fields ),*
        }
    }
}

fn build_unnamed_struct(name: &Ident, helper_name: &Ident, fields: &Punctuated<Field, Comma>) -> HelperData {
    let helper_def = unnamed_def(helper_name, fields);

    let helper_def = quote! {
        struct #helper_def;
    };

    let init_from_helper = init_from_unnamed(name, fields);

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn unnamed_def(name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|field| {
        let ty = &field.ty;
        quote! { #ty }
    });
    quote! {
        #name(#( #fields ),*)
    }
}

fn init_from_unnamed(name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let init_fields = fields.iter().enumerate().map(|(i, _)| {
        let index = Index::from(i);
        quote! { helper.#index }
    });

    quote! {
        #name(#( #init_fields ),*)
    }
}

fn build_unit_struct(name: &Ident, helper_name: &Ident) -> HelperData {
    let helper_def = quote! {
        struct #helper_name;
    };

    let init_from_helper = quote! {
        #name
    };

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn enum_def(name: &Ident, variants: &Punctuated<Variant, Comma>) -> proc_macro2::TokenStream {
    let variants = variants.iter().map(|variant| {
        let name = &variant.ident;
        match variant.fields {
            Fields::Named(ref fields) => named_def(name, &fields.named),
            Fields::Unnamed(ref fields) => unnamed_def(name, &fields.unnamed),
            Fields::Unit => quote! { #name },
        }
    });
    quote! {
        enum #name {
            #( #variants ),*
        }
    }
}

fn build_enum(name: &Ident, helper_name: &Ident, variants: &Punctuated<Variant, Comma>) -> HelperData {
    let helper_def = enum_def(helper_name, variants);

    let init_from_helper = init_from_enum(name, helper_name, variants);

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn init_from_enum(name: &Ident, helper_name: &Ident, variants: &Punctuated<Variant, Comma>) -> proc_macro2::TokenStream {
    let init_variants = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        match variant.fields {
            Fields::Named(ref fields) => init_enum_from_named(name, helper_name, variant_name, &fields.named),
            Fields::Unnamed(ref fields) => init_enum_from_unnamed(name, helper_name, variant_name, &fields.unnamed),
            Fields::Unit => init_enum_from_unit(name, helper_name, variant_name),
        }
    });
    quote! {
        match helper {
            #( #init_variants ),*
        }
    }
}

fn init_enum_from_named(name: &Ident, helper_name: &Ident, variant_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|field| {
        let name = &field.ident;
        quote! { #name }
    });
    
    let fields_clone = fields.clone();

    quote! {
        #helper_name::#variant_name { #( #fields ),* } => #name::#variant_name { #( #fields_clone ),* }
    }
}

fn init_enum_from_unnamed(name: &Ident, helper_name: &Ident, variant_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.iter().enumerate().map(|(i, _)| {
        let index = Index::from(i);
        let name = Ident::new(&format!("value_{}", i), index.span);
        quote! { #name }
    });
    
    let fields_clone = fields.clone();
    
    quote! {
        #helper_name::#variant_name( #( #fields ),* ) => #name::#variant_name( #( #fields_clone ),* )
    }
}

fn init_enum_from_unit(name: &Ident, helper_name: &Ident, variant_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #helper_name::#variant_name => #name::#variant_name
    }
}
