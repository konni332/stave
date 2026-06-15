use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
}

#[methods]
impl Widget {
    #[sets(name)]
    fn sets_name(&self, value: String) -> String {
        value
    }
}

fn main() {}
