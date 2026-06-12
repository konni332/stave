use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Block, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, ReturnType, Token, parse::Parse,
    punctuated::Punctuated,
};

use crate::common::{hidden_field_ident, no_ident, param_ident, with_ident};

struct SetsArgs {
    field: Ident,
    label: Option<Ident>,
}

impl Parse for SetsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;
        let mut label = None;
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            let kw: Ident = input.parse()?;
            if kw != "as" {
                return Err(syn::Error::new_spanned(kw, "expected `as`"));
            }
            let _: Token![=] = input.parse()?;
            label = Some(input.parse()?);
        }
        Ok(SetsArgs { field, label })
    }
}

struct RequiresArgs {
    deps: Vec<Ident>,
}

impl Parse for RequiresArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let deps = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(RequiresArgs {
            deps: deps.into_iter().collect(),
        })
    }
}

enum MethodKind {
    Sets { field: Ident, label: Option<Ident> },
    Requires { deps: Vec<Ident> },
    Plain,
}

pub(crate) struct AnnotatedMethod {
    kind: MethodKind,
    method: ImplItemFn,
}

pub struct MethodsInput {
    struct_name: Ident,
    methods: Vec<AnnotatedMethod>,
    /// Required field names inferred from #[sets] annotations
    required_fields: Vec<Ident>,
    /// Optional field names inferred from #[sets] annotations using `as =` labels.
    optional_fields: Vec<Ident>,
}

impl MethodsInput {
    pub fn parse(mut item: ItemImpl) -> syn::Result<Self> {
        let struct_name = match &*item.self_ty {
            syn::Type::Path(tp) => tp.path.get_ident().cloned().ok_or_else(|| {
                syn::Error::new_spanned(&item.self_ty, "expected a simple struct name")
            })?,
            _ => {
                return Err(syn::Error::new_spanned(
                    &item.self_ty,
                    "expected a simple struct name",
                ));
            }
        };

        let mut optional_fields: Vec<Ident> = vec![];
        let mut impl_attrs_retain = vec![];
        for attr in item.attrs.drain(..) {
            if attr.path().is_ident("optional_fields") {
                let args =
                    attr.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)?;
                optional_fields.extend(args);
            } else {
                impl_attrs_retain.push(attr);
            }
        }
        item.attrs = impl_attrs_retain;

        let mut methods = vec![];
        let mut required_fields: Vec<Ident> = vec![];

        for impl_item in item.items {
            let ImplItem::Fn(mut method) = impl_item else {
                continue;
            };
            let kind = extract_kind(&mut method)?;

            match &kind {
                MethodKind::Sets { field, .. } => {
                    if !required_fields.iter().any(|f| f == field) {
                        required_fields.push(field.clone());
                    }
                }
                MethodKind::Requires { deps } => {
                    for dep in deps {
                        if !required_fields.iter().any(|f| f == dep) {
                            required_fields.push(dep.clone());
                        }
                    }
                }
                MethodKind::Plain => {}
            }

            methods.push(AnnotatedMethod { kind, method });
        }

        Ok(MethodsInput {
            struct_name,
            methods,
            required_fields,
            optional_fields,
        })
    }
}

fn extract_kind(method: &mut ImplItemFn) -> syn::Result<MethodKind> {
    let mut sets_attr: Option<SetsArgs> = None;
    let mut requires_attr: Option<RequiresArgs> = None;
    let mut retain = vec![];

    for attr in method.attrs.drain(..) {
        if attr.path().is_ident("sets") {
            sets_attr = Some(attr.parse_args::<SetsArgs>()?);
        } else if attr.path().is_ident("requires") {
            requires_attr = Some(attr.parse_args::<RequiresArgs>()?);
        } else {
            retain.push(attr);
        }
    }
    method.attrs = retain;

    match (sets_attr, requires_attr) {
        (Some(_), Some(_)) => Err(syn::Error::new_spanned(
            &method.sig.ident,
            "a method cannot have both #[sets] and #[requires]",
        )),
        (Some(s), None) => Ok(MethodKind::Sets {
            field: s.field,
            label: s.label,
        }),
        (None, Some(r)) => Ok(MethodKind::Requires { deps: r.deps }),
        (None, None) => Ok(MethodKind::Plain),
    }
}

pub fn expand_methods(input: MethodsInput) -> syn::Result<TokenStream> {
    let struct_name = &input.struct_name;
    let required_fields = &input.required_fields;
    let optional_fields = &input.optional_fields;

    let mut output = TokenStream::new();

    for annotated in &input.methods {
        let ts = match &annotated.kind {
            MethodKind::Sets { field, label } => expand_sets_method(
                struct_name,
                required_fields,
                optional_fields,
                field,
                label.as_ref(),
                &annotated.method,
            )?,
            MethodKind::Requires { deps } => {
                expand_requires_method(struct_name, required_fields, deps, &annotated.method)?
            }
            MethodKind::Plain => {
                let method = &annotated.method;
                let params: Vec<Ident> = required_fields.iter().map(param_ident).collect();
                quote! {
                    #[allow(non_camel_case_types)]
                    impl < #(#params),* > #struct_name < #(#params),* > {
                        #method
                    }
                }
            }
        };
        output.extend(ts);
    }

    Ok(output)
}

fn expand_sets_method(
    struct_name: &Ident,
    required_fields: &[Ident],
    optional_fields: &[Ident],
    sets_field: &Ident,
    _label: Option<&Ident>,
    method: &ImplItemFn,
) -> syn::Result<TokenStream> {
    let vis = &method.vis;
    let sig = &method.sig;
    let method_name = &sig.ident;
    let body: &Block = &method.block;

    let free_params: Vec<Ident> = required_fields
        .iter()
        .filter(|f| *f != sets_field)
        .map(param_ident)
        .collect();

    let input_state: Vec<TokenStream> = required_fields
        .iter()
        .map(|f| {
            if f == sets_field {
                let no = no_ident(f);
                quote! { #no }
            } else {
                let p = param_ident(f);
                quote! { #p }
            }
        })
        .collect();

    let output_state: Vec<TokenStream> = required_fields
        .iter()
        .map(|f| {
            if f == sets_field {
                let with = with_ident(f);
                quote! { #with }
            } else {
                let p = param_ident(f);
                quote! { #p }
            }
        })
        .collect();

    let return_ty = quote! { #struct_name < #(#output_state),* > };

    let user_params: Vec<&FnArg> = sig
        .inputs
        .iter()
        .filter(|a| !matches!(a, FnArg::Receiver(_)))
        .collect();
    let carry_required: Vec<TokenStream> = required_fields
        .iter()
        .filter(|f| *f != sets_field)
        .map(|f| {
            let h = hidden_field_ident(f);
            quote! { #h: self.#h, }
        })
        .collect();
    let carry_optional: Vec<TokenStream> = optional_fields
        .iter()
        .map(|f| quote! { #f: self.#f, })
        .collect();

    let with = with_ident(sets_field);
    let hidden_sets = hidden_field_ident(sets_field);

    let impl_generics = if free_params.is_empty() {
        quote! {}
    } else {
        quote! { < #(#free_params),* > }
    };

    Ok(quote! {
        #[allow(non_camel_case_types)]
        impl #impl_generics #struct_name < #(#input_state),* > {
            #vis fn #method_name(self, #(#user_params),*) -> #return_ty {
                let __stave_value = { #body };
                #struct_name {
                    #hidden_sets: #with(__stave_value),
                    #(#carry_required)*
                    #(#carry_optional)*
                }
            }
        }
    })
}

fn expand_requires_method(
    struct_name: &Ident,
    required_fields: &[Ident],
    deps: &[Ident],
    method: &ImplItemFn,
) -> syn::Result<TokenStream> {
    for dep in deps {
        if !required_fields.iter().any(|f| f == dep) {
            return Err(syn::Error::new_spanned(
                dep,
                format!(
                    "unknown field or label `{}` in #[requires]; \
                        must match a field annotated #[stave(required)] on the builder struct \
                        or a label coined with #[sets(..., as = label)]",
                    dep
                ),
            ));
        }
    }

    let free_params: Vec<Ident> = required_fields
        .iter()
        .filter(|f| !deps.iter().any(|d| d == *f))
        .map(param_ident)
        .collect();

    let state: Vec<TokenStream> = required_fields
        .iter()
        .map(|f| {
            if deps.iter().any(|d| d == f) {
                let w = with_ident(f);
                quote! { #w }
            } else {
                let p = param_ident(f);
                quote! { #p }
            }
        })
        .collect();

    let impl_generics = if free_params.is_empty() {
        quote! {}
    } else {
        quote! { < #(#free_params),* > }
    };

    let method = rewrite_self_return(method, struct_name, &state);

    Ok(quote! {
        #[allow(non_camel_case_types)]
        impl #impl_generics #struct_name < #(#state),* > {
            #method
        }
    })
}

fn rewrite_self_return(
    method: &ImplItemFn,
    struct_name: &Ident,
    state: &[TokenStream],
) -> ImplItemFn {
    let mut m = method.clone();
    if let ReturnType::Type(arrow, ty) = &method.sig.output
        && let syn::Type::Path(tp) = ty.as_ref()
        && tp.path.is_ident("Self")
    {
        let concrete: syn::Type = syn::parse_quote! { #struct_name < #(#state),* > };
        m.sig.output = ReturnType::Type(*arrow, Box::new(concrete));
    }
    m
}

pub fn parse_and_expand(item: ItemImpl) -> syn::Result<TokenStream> {
    let input = MethodsInput::parse(item)?;
    expand_methods(input)
}

