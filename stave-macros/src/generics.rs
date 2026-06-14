use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    GenericParam, Generics, Ident, Type,
    visit::{self, Visit},
};

/// Collects the names of every type/const parameter and lifetime referenced anywhere within a type
/// (including e.g. array length expressions, so const generics used as array lenghts are picked up
/// too).
#[derive(Default)]
struct UsedNames {
    idents: HashSet<Ident>,
    lifetimes: HashSet<Ident>,
}

impl<'ast> Visit<'ast> for UsedNames {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none()
            && let Some(first) = path.segments.first()
        {
            self.idents.insert(first.ident.clone());
        }
        visit::visit_path(self, path);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
        self.lifetimes.insert(lifetime.ident.clone());
    }
}

/// Returns the subset of `generics`'s own parameters that `ty` refers to, in declaration order.
/// Useful for parametarizing a marker type that wraps a value of this type. It only needs to be
/// generic over what it actually stores.
pub fn params_used_by(generics: &Generics, ty: &Type) -> Vec<GenericParam> {
    let mut used = UsedNames::default();
    used.visit_type(ty);

    generics
        .params
        .iter()
        .filter(|param| match param {
            GenericParam::Lifetime(lt) => used.lifetimes.contains(&lt.lifetime.ident),
            GenericParam::Type(ty) => used.idents.contains(&ty.ident),
            GenericParam::Const(c) => used.idents.contains(&c.ident),
        })
        .cloned()
        .collect()
}

pub fn as_argument(param: &GenericParam) -> TokenStream {
    match param {
        GenericParam::Lifetime(lt) => lt.lifetime.to_token_stream(),
        GenericParam::Type(ty) => ty.ident.to_token_stream(),
        GenericParam::Const(c) => c.ident.to_token_stream(),
    }
}

pub fn strip_bounds(param: GenericParam) -> GenericParam {
    match param {
        GenericParam::Lifetime(lt) => GenericParam::Lifetime(syn::LifetimeParam {
            attrs: Vec::new(),
            lifetime: lt.lifetime,
            colon_token: None,
            bounds: Default::default(),
        }),
        GenericParam::Type(ty) => GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: ty.ident,
            colon_token: None,
            bounds: Default::default(),
            eq_token: None,
            default: None,
        }),
        GenericParam::Const(c) => GenericParam::Const(syn::ConstParam {
            attrs: Vec::new(),
            const_token: c.const_token,
            ident: c.ident,
            colon_token: c.colon_token,
            ty: c.ty,
            eq_token: None,
            default: None,
        }),
    }
}
