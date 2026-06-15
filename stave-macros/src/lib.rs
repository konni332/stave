use proc_macro::TokenStream;
use syn::{ItemImpl, ItemStruct, parse_macro_input};

mod builder;
mod common;
mod generics;
mod methods;
mod registry;

/// Turns a plain struct into a typestate builder.
///
/// Fields can be annotated with `#[stave(required)]` or
/// `#[stave(optional)]`. Fields without either annotation are treated as
/// optional. Optional fields are simply wrapped in `Option<T>`.
///
/// For each required field, `#[builder]` generates two hidden marker types -
/// one representing the field being unset, one representing it being set and
/// carrying the field's value - and adds a corresponding generic parameter to
/// the struct. A field's value can only be read once its marker type proves
/// it has been set; see the `#[methods]` macro for how that state is
/// threaded through and queried.
///
/// A `new()` constructor is generated, returning the struct with every
/// required field in its unset state and every optional field set to `None`.
///
/// ```rust,ignore
/// #[builder]
/// struct Server {
///     #[stave(required)]
///     host: String,
///     #[stave(required)]
///     port: u16,
///     #[stave(optional)]
///     timeout: Duration,
/// }
///
/// let server = Server::new();
/// ```
///
/// The struct's own generic parameters (lifetimes, type parameters, const
/// parameters) are preserved, and the generated marker types are themselves
/// parameterized over whichever of those parameters the corresponding field's
/// type actually uses.
#[proc_macro_attribute]
pub fn builder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    builder::expand(input)
        .unwrap_or_else(|err| err.write_errors())
        .into()
}

#[proc_macro_attribute]
pub fn methods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    methods::expand(input)
        .unwrap_or_else(|err| err.write_errors())
        .into()
}
