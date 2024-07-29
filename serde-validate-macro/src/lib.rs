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
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Index, Variant, Generics, GenericParam};
use quote::{quote, ToTokens};
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
    
    let generics = &input.generics;

    let helper_name = Ident::new(&format!("__ValidDeserialize{name}"), name.span());

    let HelperData { helper_def, init_from_helper } = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => build_named_struct(&name, &helper_name, &generics, &fields.named),
            Fields::Unnamed(ref fields) => build_unnamed_struct(&name, &helper_name, &generics, &fields.unnamed),
            Fields::Unit => build_unit_struct(&name, &helper_name),
        }
        Data::Enum(ref data) => build_enum(&name, &helper_name, &generics, &data.variants),
        Data::Union(_) => {unimplemented!()}
    };
    
    let generic_params = generics.params.to_token_stream();
    let extra_where_clause: Vec<_> = generics.params.iter().filter_map(|p| match p {
        GenericParam::Type(p) => {
            let p = &p.ident;
            Some(quote! { #p : serde::Deserialize<'__de> })
        }
        _ => None,
    }).collect();
    let where_clause = match generics.where_clause {
        None => quote! {
            where #(#extra_where_clause,)*
        },
        Some(ref clause) => {
            let predicates = clause.predicates.iter().map(|p| p.to_token_stream());
            quote! {
                where #(#predicates,)* #(#extra_where_clause,)*
            }
        }
    };
    let simple_gen_params = generics.params.iter().map(|p| match p {
        GenericParam::Type(p) => {
            let p = &p.ident;
            quote! { #p }
        }
        GenericParam::Lifetime(p) => {
            let p = &p.lifetime;
            quote! { #p }
        },
        GenericParam::Const(p) => {
            let p = &p.ident;
            quote! { #p }
        },
    });

    let tokens = quote! {
        #input

        #[derive(serde::Deserialize)]
        #helper_def

        impl <'__de, #generic_params> serde::Deserialize<'__de> for #name<#(#simple_gen_params,)*> #where_clause {
            fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: serde::Deserializer<'__de>
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

fn build_named_struct(name: &Ident, helper_name: &Ident, generics: &Generics, fields: &Punctuated<Field, Comma>) -> HelperData {
    let helper_def = named_def_full(helper_name, generics, fields);

    let helper_def = quote! {
        struct #helper_def
    };

    let init_from_helper = init_from_named(name, fields);

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn named_def_full(name: &Ident, generics: &Generics, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let generic_params = generics.params.to_token_stream();
    let where_clause = generics.where_clause.to_token_stream();
    let fields = fields.to_token_stream();
    quote! {
        #name<#generic_params> #where_clause {
            #fields
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

fn build_unnamed_struct(name: &Ident, helper_name: &Ident, generics: &Generics, fields: &Punctuated<Field, Comma>) -> HelperData {
    let helper_def = unnamed_def_full(helper_name, generics, fields);

    let helper_def = quote! {
        struct #helper_def
    };

    let init_from_helper = init_from_unnamed(name, fields);

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn unnamed_def_full(name: &Ident, generics: &Generics, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let generic_params = generics.params.to_token_stream();
    let where_clause = generics.where_clause.to_token_stream();
    let fields = fields.to_token_stream();
    quote! {
        #name<#generic_params>(#fields) #where_clause;
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

fn build_enum(name: &Ident, helper_name: &Ident, generics: &Generics, variants: &Punctuated<Variant, Comma>) -> HelperData {
    let helper_def = enum_def(helper_name, generics, variants);

    let init_from_helper = init_from_enum(name, helper_name, variants);

    HelperData {
        helper_def,
        init_from_helper
    }
}

fn enum_def(name: &Ident, generics: &Generics, variants: &Punctuated<Variant, Comma>) -> proc_macro2::TokenStream {
    let generic_params = generics.params.to_token_stream();
    let where_clause = generics.where_clause.to_token_stream();
    let variants = variants.iter().map(|variant| {
        let name = &variant.ident;
        match variant.fields {
            Fields::Named(ref fields) => named_def(name, &fields.named),
            Fields::Unnamed(ref fields) => unnamed_def(name, &fields.unnamed),
            Fields::Unit => quote! { #name },
        }
    });
    quote! {
        enum #name<#generic_params> #where_clause {
            #( #variants ),*
        }
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

fn named_def(name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.to_token_stream();
    quote! {
        #name {
            #fields
        }
    }
}

fn unnamed_def(name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.to_token_stream();
    quote! {
        #name(#fields)
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
