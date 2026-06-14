use darling::FromField;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, GenericParam, Generics, Ident, ItemStruct, Type, TypeParam};

use crate::generics::{as_argument, params_used_by, strip_bounds};

#[derive(FromField)]
#[darling(attributes(stave))]
struct FieldAttrs {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    required: darling::util::Flag,
    #[darling(default)]
    optional: darling::util::Flag,
}

/// A required field, together with the identifiers stave generates for it.
struct RequiredField {
    /// Original field type, e.g. `String`
    ty: Type,
    /// `__FooUnset`
    unset: Ident,
    /// `__FooSet`
    set: Ident,
    /// `__FooState`
    state: Ident,
    /// `__stave_foo`
    storage: Ident,
    /// The subset of the structs own generic parameters that `ty` refers to, stripped of
    /// bounds/defaults. These parameterize `set`.
    marker_params: Vec<GenericParam>,
}

/// An optional field, kept as-is but wrapped in Option<T>.
struct OptionalField {
    name: Ident,
    ty: Type,
}

pub fn expand(input: ItemStruct) -> darling::Result<TokenStream> {
    let fields = match &input.fields {
        Fields::Named(fields) => &fields.named,
        other => {
            return Err(darling::Error::custom(
                "#[builder] can only be applied to structs with named fields",
            )
            .with_span(other));
        }
    };

    let mut required = Vec::new();
    let mut optional = Vec::new();
    let mut errors = darling::Error::accumulator();

    for field in fields {
        let Some(attrs) = errors.handle(FieldAttrs::from_field(field)) else {
            continue;
        };
        let name = attrs.ident.expect("named field always has an ident");

        match (attrs.required.is_present(), attrs.optional.is_present()) {
            (true, false) => required.push(make_required_field(&input.generics, &name, attrs.ty)),
            (false, true) => optional.push(OptionalField { name, ty: attrs.ty }),
            (true, true) => errors.push(
                darling::Error::custom("a field cannot be both `required` and `optional`")
                    .with_span(&name),
            ),
            (false, false) => optional.push(OptionalField { name, ty: attrs.ty }),
        }
    }
    errors.finish()?;

    Ok(generate(&input, &required, &optional))
}

fn make_required_field(generics: &Generics, name: &Ident, ty: Type) -> RequiredField {
    let pascal = name.to_string().to_pascal_case();
    let marker_params = params_used_by(generics, &ty)
        .into_iter()
        .map(strip_bounds)
        .collect();

    RequiredField {
        ty,
        unset: format_ident!("__{pascal}Unset"),
        set: format_ident!("__{pascal}Set"),
        state: format_ident!("__{pascal}State"),
        storage: format_ident!("__stave_{name}"),
        marker_params,
    }
}

fn generate(
    input: &ItemStruct,
    required: &[RequiredField],
    optional: &[OptionalField],
) -> TokenStream {
    let vis = &input.vis;
    let ident = &input.ident;
    let attrs = &input.attrs;
    let where_clause = &input.generics.where_clause;

    let markers = required.iter().map(|f| {
        let RequiredField {
            ty,
            unset,
            set,
            marker_params,
            ..
        } = f;
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types, dead_code)]
            pub(crate) struct #unset;

            #[doc(hidden)]
            #[allow(non_camel_case_types, dead_code)]
            pub(crate) struct #set<#(#marker_params),*>(#ty);
        }
    });

    let state_params = required.iter().map(|f| {
        GenericParam::Type(TypeParam {
            attrs: Vec::new(),
            ident: f.state.clone(),
            colon_token: None,
            bounds: Default::default(),
            eq_token: None,
            default: None,
        })
    });

    let struct_params = input.generics.params.iter().cloned().chain(state_params);

    let storage_fields = required.iter().map(|f| {
        let RequiredField { storage, state, .. } = f;
        quote! { #storage: #state }
    });

    let optional_fields = optional.iter().map(|f| {
        let OptionalField { name, ty } = f;
        quote! { #name: ::core::option::Option<#ty> }
    });

    // the structs own generic params don't necessarily appear in any of its own fields (they may
    // only appear inside `__FooSet<...>`, which `Self` never names directly). A `PhantomData`
    // field "uses" each of them so the compiler does not complain.
    let phantom_field = phantom_field(&input.generics);
    let phantom_init = phantom_field
        .is_some()
        .then(|| quote! { __stave_phantom: ::core::marker::PhantomData });

    let new_impl_params = &input.generics.params;
    let new_struct_args = input
        .generics
        .params
        .iter()
        .map(as_argument)
        .chain(required.iter().map(|f| {
            let unset = &f.unset;
            quote! { #unset }
        }));

    let storage_inits = required.iter().map(|f| {
        let RequiredField { storage, unset, .. } = f;
        quote! { #storage: #unset }
    });

    let optional_inits = optional.iter().map(|f| {
        let name = &f.name;
        quote! { #name: ::core::option::Option::None }
    });

    quote! {
        #(#markers)*

        #(#attrs)*
        #vis struct #ident <#(#struct_params),*> #where_clause {
            #(#storage_fields,)*
            #(#optional_fields,)*
            #phantom_field
        }

        impl <#new_impl_params> #ident <#(#new_struct_args),*> #where_clause {
            pub fn new() -> Self {
                #ident {
                    #(#storage_inits,)*
                    #(#optional_inits,)*
                    #phantom_init
                }
            }
        }
    }
}

fn phantom_field(generics: &Generics) -> Option<TokenStream> {
    if generics.params.is_empty() {
        return None;
    }

    let phantom_types = generics.params.iter().map(|param| match param {
        GenericParam::Lifetime(lt) => {
            let lifetime = &lt.lifetime;
            quote! { &#lifetime () }
        }
        GenericParam::Type(ty) => {
            let ident = &ty.ident;
            quote! { #ident }
        }
        GenericParam::Const(c) => {
            let ident = &c.ident;
            quote! { [(); #ident] }
        }
    });

    Some(quote! {
        __stave_phantom: ::core::marker::PhantomData<(#(#phantom_types),*)>
    })
}
