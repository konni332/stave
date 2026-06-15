use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
    note: String,
}

#[methods]
impl Widget {
    #[requires(note)]
    fn go(&self) {}
}

fn main() {}
