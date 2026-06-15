mod builder;
mod methods;
mod methods_generics;

#[allow(dead_code)]
// this main needs to be here, because otherwise trybuild complains.
// This mod file is not needed, but made debugging easier, because it provides a better LSP DX
fn main() {}
