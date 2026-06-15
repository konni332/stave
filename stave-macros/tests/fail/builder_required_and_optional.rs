use stave_macros::builder;

#[builder]
struct Widget {
    #[stave(required, optional)]
    name: String,
}

fn main() {}
