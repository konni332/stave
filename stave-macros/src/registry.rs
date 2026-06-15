use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

pub type Tokens = String;

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: Tokens,
}

#[derive(Debug, Clone)]
pub struct RequiredFieldInfo {
    pub name: String,
    pub ty: Tokens,
    /// The structs own generic parameters (with bounds) that this fields marker types are
    /// parameterized over, e.g. `T, const N: usize`.
    #[allow(dead_code)] // for some reason the compiler does not pickup on its usage in the
    // `quote!` macro
    pub marker_params: Tokens,
    /// The same parameters in argument position, e.g. `T, N`.
    pub marker_args: Tokens,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    /// The structs own generic parameters, with bounds, e.g. `'a, T: Clone, const N: usize`.
    pub generic_params: Tokens,
    pub generic_args: Tokens,
    /// The structs `where` clause, if any, or empty
    pub where_clause: Tokens,
    pub required: Vec<RequiredFieldInfo>,
    pub optional: Vec<FieldInfo>,
}

fn registry() -> &'static Mutex<HashMap<String, StructInfo>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, StructInfo>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Called by `#[builder]` once it has analyzed a struct, so that a later `#[methods]` block for
/// the same struct can reconstruct its typestate layout without the user restating anything.
pub fn register(struct_name: String, info: StructInfo) {
    registry().lock().unwrap().insert(struct_name, info);
}

/// Called by `#[methods]` to retrieve what `#[builder]` recorded for `struct_name`.
/// `None` means no `#[builder]`-annotated struct with this name has been expanded yet.
/// In practice, `#[builder]` is missing or appears later in the source than this `#[methods]`
/// block.
///
/// # NOTE
/// For now this should emit a compile error, but this may change in the future.
pub fn lookup(struct_name: &str) -> Option<StructInfo> {
    registry().lock().unwrap().get(struct_name).cloned()
}
