use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
}

#[methods]
impl Widget {
    const VERSION: u32 = 1;
}

fn main() {}
