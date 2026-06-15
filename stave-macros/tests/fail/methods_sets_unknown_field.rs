use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
}

#[methods]
impl Widget {
    #[sets(missing)]
    fn go(self) -> String {
        String::new()
    }
}

fn main() {}
