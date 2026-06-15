use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
}

#[methods]
impl Widget {
    #[sets(name)]
    #[requires(name)]
    fn go(self, value: String) -> String {
        value
    }
}

fn main() {}
