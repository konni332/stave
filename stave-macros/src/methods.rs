use std::collections::{HashMap, HashSet};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Block, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Type, punctuated::Punctuated,
    token::Comma,
};

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

    errors.finish()?;

    for field in &fields.required {
        if !covered.contains(field.name.to_string().as_str()) {
            impls.push(auto_required_setter(&self_ident, &fields, field));
        }
        impls.push(auto_required_getter(&self_ident, &fields, field));
    }
    for field in &fields.optional {
        if !covered.contains(field.name.to_string().as_str()) {
            impls.push(auto_optional_setter(&self_ident, &fields, field));
        }
        impls.push(auto_optional_getter(&self_ident, &fields, field));
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

/// Builds a fixed-state map with every field in `requires` pinned to `Set`.
fn build_fixed(
    fields: &Fields,
    requires: &[Ident],
) -> darling::Result<HashMap<String, FixedState>> {
    let mut fixed = HashMap::new();
    for name in requires {
        validate_required_field(fields, name)?;
        if fixed.insert(name.to_string(), FixedState::Set).is_some() {
            return Err(
                darling::Error::custom(format!("`{name}` is listed twice in `requires`"))
                    .with_span(name),
            );
        }
    }
    Ok(fixed)
}

/// Constructs a `Server {...}` expression that moves every field across from `self` unchanged,
/// except `target`, whose storage field is initialized from `value_expr`.
fn rebuild_self(
    self_ident: &Ident,
    fields: &Fields,
    target: &RequiredField,
    value_expr: TokenStream,
) -> TokenStream {
    let mut inits = Vec::new();

    for field in &fields.required {
        let storage = &field.storage;
        if field.name == target.name {
            inits.push(quote! { #storage: #value_expr });
        } else {
            inits.push(quote! { #storage: self.#storage });
        }
    }

    for field in &fields.optional {
        let name = &field.name;
        inits.push(quote! { #name: self.#name });
    }

    if fields.has_phantom {
        inits.push(quote! { __stave_phantom: self.__stave_phantom });
    }

    quote! { #self_ident { #(#inits),* } }
}

fn empty_block() -> Block {
    syn::parse2(quote! {{}}).expect("empty block is always valid")
}

fn ensure_takes_self_by_value(sig: &syn::Signature) -> darling::Result<()> {
    match sig.inputs.first() {
        Some(FnArg::Receiver(receiver)) if receiver.reference.is_none() => Ok(()),
        Some(arg) => Err(darling::Error::custom(
            "a `sets` method must take `self` by value (`self` or `mut self`), \
                since it consumes the builder to change its type",
        )
        .with_span(arg)),
        None => {
            Err(darling::Error::custom("a `sets` method must take `self`").with_span(&sig.ident))
        }
    }
}

fn take_sets_attr(attrs: &mut Vec<Attribute>) -> darling::Result<Option<Ident>> {
    let mut result = None;
    let mut remaining = Vec::new();

    for attr in attrs.drain(..) {
        if attr.path().is_ident("sets") {
            if result.is_some() {
                return Err(darling::Error::custom(
                    "multiple `#[sets(...)]` attributes on the same method",
                )
                .with_span(&attr));
            }
            result = Some(attr.parse_args::<Ident>().map_err(darling::Error::from)?);
        } else {
            remaining.push(attr);
        }
    }

    *attrs = remaining;
    Ok(result)
}

fn take_requires_attr(attrs: &mut Vec<Attribute>) -> darling::Result<Vec<Ident>> {
    let mut result = Vec::new();
    let mut remaining = Vec::new();

    for attr in attrs.drain(..) {
        if attr.path().is_ident("requires") {
            let idents = attr
                .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                .map_err(darling::Error::from)?;
            result.extend(idents);
        } else {
            remaining.push(attr);
        }
    }

    *attrs = remaining;
    Ok(result)
}

fn generate_method(
    mut method: ImplItemFn,
    fields: &Fields,
    self_ident: &Ident,
    covered: &mut HashSet<String>,
) -> darling::Result<TokenStream> {
    let sets = take_sets_attr(&mut method.attrs)?;
    let requires = take_requires_attr(&mut method.attrs)?;

    match sets {
        Some(field_name) => {
            covered.insert(field_name.to_string());

            if let Some(field) = fields.required.iter().find(|f| f.name == field_name) {
                requires_setter(method, fields, self_ident, field, &requires)
            } else if let Some(field) = fields.optional.iter().find(|f| f.name == field_name) {
                optional_setter(method, fields, self_ident, field, &requires)
            } else {
                Err(
                    darling::Error::custom(format!("`{field_name}` is not a field of this struct"))
                        .with_span(&field_name),
                )
            }
        }
        None if !requires.is_empty() => requires_method(method, fields, self_ident, &requires),
        None => plain_method(method, fields, self_ident),
    }
}

fn plain_method(
    method: ImplItemFn,
    fields: &Fields,
    self_ident: &Ident,
) -> darling::Result<TokenStream> {
    let generics = impl_generics(fields, &HashSet::new());
    let self_ty = self_type_for(self_ident, fields, &HashMap::new());
    let where_clause = &fields.where_clause;

    Ok(quote! {
        impl<#generics> #self_ty #where_clause {
            #method
        }
    })
}

fn requires_method(
    method: ImplItemFn,
    fields: &Fields,
    self_ident: &Ident,
    requires: &[Ident],
) -> darling::Result<TokenStream> {
    let fixed = build_fixed(fields, requires)?;
    let fixed_names: HashSet<String> = fixed.keys().cloned().collect();

    let generics = impl_generics(fields, &fixed_names);
    let self_ty = self_type_for(self_ident, fields, &fixed);
    let where_clause = &fields.where_clause;

    Ok(quote! {
        impl<#generics> #self_ty #where_clause {
            #method
        }
    })
}

fn requires_setter(
    mut method: ImplItemFn,
    fields: &Fields,
    self_ident: &Ident,
    field: &RequiredField,
    requires: &[Ident],
) -> darling::Result<TokenStream> {
    if requires.iter().any(|name| name == &field.name) {
        return Err(darling::Error::custom(format!(
            "`{}` cannot appear in both sets and `requires` on the same method",
            field.name
        ))
        .with_span(&field.name));
    }

    ensure_takes_self_by_value(&method.sig)?;

    let mut fixed = build_fixed(fields, requires)?;
    let mut fixed_names: HashSet<String> = fixed.keys().cloned().collect();
    fixed_names.insert(field.name.to_string());

    let generics = impl_generics(fields, &fixed_names);

    fixed.insert(field.name.to_string(), FixedState::Unset);
    let input_self = self_type_for(self_ident, fields, &fixed);

    fixed.insert(field.name.to_string(), FixedState::Set);
    let output_self = self_type_for(self_ident, fields, &fixed);

    let field_ty = &field.ty;
    let set = &field.set;
    let body = std::mem::replace(&mut method.block, empty_block());
    let rebuild = rebuild_self(self_ident, fields, field, quote! { #set(__stave_value) });

    method.sig.output = syn::ReturnType::Type(
        Default::default(),
        Box::new(syn::parse2(output_self).map_err(darling::Error::from)?),
    );
    method.block = syn::parse2(quote! {{
        let __stave_value: #field_ty = #body;
        #rebuild
    }})
    .map_err(darling::Error::from)?;

    let where_clause = &fields.where_clause;

    Ok(quote! {
        impl <#generics> #input_self #where_clause {
            #method
        }
    })
}

fn optional_setter(
    mut method: ImplItemFn,
    fields: &Fields,
    self_ident: &Ident,
    field: &OptionalField,
    requires: &[Ident],
) -> darling::Result<TokenStream> {
    ensure_takes_self_by_value(&method.sig)?;

    let fixed = build_fixed(fields, requires)?;
    let fixed_names: HashSet<String> = fixed.keys().cloned().collect();

    let generics = impl_generics(fields, &fixed_names);
    let self_ty = self_type_for(self_ident, fields, &fixed);

    let field_ty = &field.ty;
    let name = &field.name;
    let body = std::mem::replace(&mut method.block, empty_block());

    method.sig.output = syn::ReturnType::Type(
        Default::default(),
        Box::new(syn::parse2(self_ty.clone()).map_err(darling::Error::from)?),
    );
    method.block = syn::parse2(quote! {{
        let __stave_value: #field_ty = #body;
        self.#name = ::core::option::Option::Some(__stave_value);
        self
    }})
    .map_err(darling::Error::from)?;

    let where_clause = &fields.where_clause;

    Ok(quote! {
        impl<#generics> #self_ty #where_clause {
            #method
        }
    })
}

fn auto_required_setter(self_ident: &Ident, fields: &Fields, field: &RequiredField) -> TokenStream {
    let fixed_names: HashSet<String> = std::iter::once(field.name.to_string()).collect();
    let generics = impl_generics(fields, &fixed_names);

    let mut fixed = HashMap::new();
    fixed.insert(field.name.to_string(), FixedState::Unset);
    let input_self = self_type_for(self_ident, fields, &fixed);

    fixed.insert(field.name.to_string(), FixedState::Set);
    let output_self = self_type_for(self_ident, fields, &fixed);

    let sets_fn = format_ident!("set_{}", field.name);
    let ty = &field.ty;
    let set = &field.set;
    let rebuild = rebuild_self(self_ident, fields, field, quote! { #set(value) });
    let where_clause = &fields.where_clause;

    quote! {
        impl<#generics> #input_self #where_clause {
            pub fn #sets_fn(self, value: #ty) -> #output_self {
                #rebuild
            }
        }
    }
}
fn auto_optional_setter(self_ident: &Ident, fields: &Fields, field: &OptionalField) -> TokenStream {
    let generics = impl_generics(fields, &HashSet::new());
    let self_ty = self_type_for(self_ident, fields, &HashMap::new());

    let sets_fn = format_ident!("set_{}", field.name);
    let ty = &field.ty;
    let name = &field.name;
    let where_clause = &fields.where_clause;

    quote! {
        impl<#generics> #self_ty #where_clause {
            pub fn #sets_fn(mut self, value: #ty) -> Self {
                self.#name = ::core::option::Option::Some(value);
                self
            }
        }
    }
}

fn auto_optional_getter(self_ident: &Ident, fields: &Fields, field: &OptionalField) -> TokenStream {
    let generics = impl_generics(fields, &HashSet::new());
    let self_ty = self_type_for(self_ident, fields, &HashMap::new());
    let where_clause = &fields.where_clause;
    let field_ty = &field.ty;
    let field_name = &field.name;
    quote! {
        impl<#generics> #self_ty #where_clause {
            pub fn #field_name(&self) -> &::core::option::Option<#field_ty> {
                &self.#field_name
            }
        }
    }
}

fn auto_required_getter(self_ident: &Ident, fields: &Fields, field: &RequiredField) -> TokenStream {
    let mut fixed = HashMap::new();
    fixed.insert(field.name.to_string(), FixedState::Set);
    let fixed_names: HashSet<String> = std::iter::once(field.name.to_string()).collect();

    let generics = impl_generics(fields, &fixed_names);
    let self_ty = self_type_for(self_ident, fields, &fixed);

    let where_clause = &fields.where_clause;

    let field_ty = &field.ty;
    let fn_name = &field.name;
    let field_name = format_ident!("__stave_{}", field.name);
    quote! {
        impl<#generics> #self_ty #where_clause {
            pub fn #fn_name(&self) -> &#field_ty {
                &self.#field_name.0
            }
        }
    }
}
