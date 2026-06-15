use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
}

#[methods]
impl Widget {
    #[requires(missing)]
    fn go(&self) {}
}

fn main() {}
