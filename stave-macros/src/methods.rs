use std::collections::{HashMap, HashSet};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ImplItem, ItemImpl, Type};

use crate::registry;

pub fn expand(input: ItemImpl) -> darling::Result<TokenStream> {
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(darling::Error::custom(
            "#[methods] impl blocks should not declare their own generic parameters; \
                #[builder] already recorded the structs generics",
        )
        .with_span(&input.generics));
    }

    if let Some((_, trait_, _)) = &input.trait_ {
        return Err(
            darling::Error::custom("#[methods] does not support trait implementations")
                .with_span(trait_),
        );
    }

    let self_ident = self_type_ident(&input.self_ty)?;

    let info = registry::lookup(&self_ident.to_string()).ok_or_else(|| {
        darling::Error::custom(format!(
            "no `#[builder]` struct `{self_ident}` was found; make sure `#[builder]` is \
                applied to `{self_ident}` and its definition appears earlier in this file than \
                this `#[methods]` blocks"
        ))
        .with_span(&input.self_ty)
    })?;

    let fields = Fields::from_info(&info);

    let mut errors = darling::Error::accumulator();
    let mut covered: HashSet<String> = HashSet::new();
    let mut impls = Vec::new();

    for item in input.items {
        let ImplItem::Fn(method) = item else {
            errors.push(
                darling::Error::custom("only methods are supported inside a #[methods] impl block")
                    .with_span(&item),
            );
            continue;
        };

        let Some(generated) =
            errors.handle(generate_method(method, &fields, &self_ident, &mut covered))
        else {
            continue;
        };
        impls.push(generated);
    }

    errors.finish();

    for field in &fields.required {
        if !covered.contains(field.name.to_string().as_str()) {
            impls.push(auto_required_setter(&self_ident, &fields, field));
        }
    }
    for field in &fields.optional {
        if !covered.contains(field.name.to_string().as_str()) {
            impls.push(auto_optional_setter(&self_ident, &fields, field));
        }
    }

    Ok(quote! { #(#impls)* })
}

fn self_type_ident(ty: &Type) -> darling::Result<Ident> {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.clone())
            .ok_or_else(|| darling::Error::custom("expected struct name").with_span(ty)),
        _ => Err(
            darling::Error::custom("#[methods] must be applied to `impl StructName { ... }`")
                .with_span(ty),
        ),
    }
}

/// A required field, with the same derived identifiers `#[builder]` generated
/// for it, reconstructed from the registry.
struct RequiredField {
    name: Ident,
    ty: TokenStream,
    unset: Ident,
    set: Ident,
    state: Ident,
    storage: Ident,
    /// The struct's own generic parameters (in argument form) that `#set` is
    /// parameterized over, e.g. `T, N`.
    marker_args: TokenStream,
}

struct OptionalField {
    name: Ident,
    ty: TokenStream,
}

struct Fields {
    required: Vec<RequiredField>,
    optional: Vec<OptionalField>,
    /// The struct's own generic parameters, with bounds.
    generic_params: TokenStream,
    /// The struct's own generic parameters, in argument form.
    generic_args: TokenStream,
    where_clause: TokenStream,
    /// Whether the struct has a `__stave_phantom` field that needs to be
    /// carried across when rebuilding `Self`.
    has_phantom: bool,
}

impl Fields {
    fn from_info(info: &registry::StructInfo) -> Self {
        let required = info
            .required
            .iter()
            .map(|field| {
                let pascal = field.name.to_pascal_case();
                RequiredField {
                    name: format_ident!("{}", field.name),
                    ty: ts(&field.ty),
                    unset: format_ident!("__{pascal}Unset"),
                    set: format_ident!("__{pascal}Set"),
                    state: format_ident!("__{pascal}State"),
                    storage: format_ident!("__stave_{}", field.name),
                    marker_args: ts(&field.marker_args),
                }
            })
            .collect();

        let optional = info
            .optional
            .iter()
            .map(|field| OptionalField {
                name: format_ident!("{}", field.name),
                ty: ts(&field.ty),
            })
            .collect();

        Fields {
            required,
            optional,
            generic_params: ts(&info.generic_params),
            generic_args: ts(&info.generic_args),
            where_clause: ts(&info.where_clause),
            has_phantom: !info.generic_params.is_empty(),
        }
    }
}

fn ts(s: &str) -> TokenStream {
    s.parse()
        .expect("token streams recorded by #[builder] are always valid")
}

#[derive(Clone, Copy)]
pub enum FixedState {
    Unset,
    Set,
}

fn join_commas(parts: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let mut out = TokenStream::new();
    for part in parts.into_iter().filter(|part| !part.is_empty()) {
        if !out.is_empty() {
            out.extend(quote! { , });
        }
        out.extend(part);
    }
    out
}

/// The `impl<...>` generic parameter list for an impl block in which the
/// required fields named in `fixed` are pinned to a concrete state; every
/// other required field gets a fresh `__FooState` parameter.
fn impl_generics(fields: &Fields, fixed: &HashSet<String>) -> TokenStream {
    let mut parts = vec![fields.generic_params.clone()];
    for field in &fields.required {
        if !fixed.contains(field.name.to_string().as_str()) {
            let state = &field.state;
            parts.push(quote! { #state });
        }
    }
    join_commas(parts)
}

/// The generic argument for a single required field, givec a fixed-state map.
fn state_arg(field: &RequiredField, fixed: &HashMap<String, FixedState>) -> TokenStream {
    match fixed.get(field.name.to_string().as_str()) {
        Some(FixedState::Unset) => {
            let unset = &field.unset;
            quote! { #unset }
        }
        Some(FixedState::Set) => {
            let set = &field.set;
            let args = &field.marker_args;
            if args.is_empty() {
                quote! { #set }
            } else {
                quote! { #set<#args> }
            }
        }
        None => {
            let state = &field.state;
            quote! { #state }
        }
    }
}

// The full `Server<...>` (or just `Server`, if it needs no args) for a given fixed-state map.
fn self_type_for(
    self_ident: &Ident,
    fields: &Fields,
    fixed: &HashMap<String, FixedState>,
) -> TokenStream {
    let args = join_commas(
        std::iter::once(fields.generic_args.clone())
            .chain(fields.required.iter().map(|field| state_arg(field, fixed))),
    );
    if args.is_empty() {
        quote! { #self_ident }
    } else {
        quote! { #self_ident<#args> }
    }
}

fn validate_required_field<'a>(
    fields: &'a Fields,
    name: &Ident,
) -> darling::Result<&'a RequiredField> {
    fields
        .required
        .iter()
        .find(|field| &field.name == name)
        .ok_or_else(|| {
            if fields.optional.iter().any(|field| &field.name == name) {
                darling::Error::custom(format!(
                    "`required` can only reference required fields, but `{name}` is optional"
                ))
                .with_span(name)
            } else {
                darling::Error::custom(format!("`{name}` is not a field of this struct"))
                    .with_span(name)
            }
        })
}

