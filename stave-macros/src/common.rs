use proc_macro2::Span;
use syn::Ident;

/// `credentials` -> `__StaveNoCredentials`
pub fn no_ident(field: &Ident) -> Ident {
    Ident::new(
        &format!("__StaveNo{}", pascal_case(&field.to_string())),
        Span::call_site(),
    )
}

/// `credentials` -> `__StaveWithCredentials`
pub fn with_ident(field: &Ident) -> Ident {
    Ident::new(
        &format!("__StaveWith{}", pascal_case(&field.to_string())),
        Span::call_site(),
    )
}

/// `credentials` -> `__StaveCredentials` (generic param name)
pub fn param_ident(field: &Ident) -> Ident {
    Ident::new(
        &format!("__Stave{}", pascal_case(&field.to_string())),
        Span::call_site(),
    )
}

/// `__stave_credentials` (hidden struct field name)
pub fn hidden_field_ident(field: &Ident) -> Ident {
    Ident::new(&format!("__stave_{}", field), Span::call_site())
}

pub fn pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

