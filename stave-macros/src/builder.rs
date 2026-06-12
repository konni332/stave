use darling::{FromDeriveInput, FromField, ast};
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Type};

use crate::common::{no_ident, param_ident, with_ident};

#[derive(Debug, FromField)]
#[darling(attributes(stave))]
struct StaveField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    required: bool,
    #[darling(default)]
    optional: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(stave), supports(struct_named))]
pub(crate) struct StaveBuilderInput {
    ident: Ident,
    vis: syn::Visibility,
    data: ast::Data<(), StaveField>,
}

pub(crate) fn expand_builder(input: StaveBuilderInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let vis = &input.vis;

    let fields = input.data.take_struct().unwrap();
    let fields = fields.fields;

    for f in &fields {
        let name = f.ident.as_ref().unwrap();
        match (f.required, f.optional) {
            (true, true) => {
                return Err(syn::Error::new_spanned(
                    name,
                    "field cannot be both #[stave(required)] and #[stave(optional)]",
                ));
            }
            (false, false) => {
                return Err(syn::Error::new_spanned(
                    name,
                    "field must be annotated with either #[stave(required)] or #[stave(optional)]",
                ));
            }
            _ => {}
        }
    }

    let required: Vec<&StaveField> = fields.iter().filter(|f| f.required).collect();
    let optional: Vec<&StaveField> = fields.iter().filter(|f| f.optional).collect();

    let state_structs = required.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let no = no_ident(field_ident);
        let with = with_ident(field_ident);
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #no;
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #with(#vis #ty);
        }
    });

    let generic_params: Vec<Ident> = required
        .iter()
        .map(|f| param_ident(f.ident.as_ref().unwrap()))
        .collect();

    let required_struct_fields = required.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let param = param_ident(field_ident);
        let hidden_ident = Ident::new(&format!("__stave_{}", field_ident), Span::call_site());
        quote! {
            #[doc(hidden)]
            #hidden_ident: #param,
        }
    });

    let optional_struct_fields = optional.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! {
            #field_ident: Option<#ty>,
        }
    });

    let struct_def = quote! {
        #[allow(non_camel_case_types)]
        #vis struct #struct_name < #(#generic_params),* > {
            #(#required_struct_fields)*
            #(#optional_struct_fields)*
        }
    };

    let no_idents: Vec<Ident> = required
        .iter()
        .map(|f| no_ident(f.ident.as_ref().unwrap()))
        .collect();

    let hidden_field_inits_no = required.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let no = no_ident(field_ident);
        let hidden_ident = Ident::new(&format!("__stave_{}", field_ident), Span::call_site());
        quote! { #hidden_ident: #no, }
    });

    let optional_field_inits_none = optional.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        quote! { #field_ident: None, }
    });

    let new_impl = quote! {
        #[allow(non_camel_case_types)]
        impl #struct_name < #(#no_idents),* > {
            pub fn new() -> Self {
                Self {
                    #(#hidden_field_inits_no)*
                    #(#optional_field_inits_none)*
                }
            }
        }
    };

    Ok(quote! {
        #(#state_structs)*
        #struct_def
        #new_impl
    })
}
