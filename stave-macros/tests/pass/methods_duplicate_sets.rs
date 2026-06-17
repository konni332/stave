use stave_macros::{builder, methods};

#[builder]
struct Widget {
    #[stave(required)]
    name: String,
}

#[methods]
impl Widget {
    #[sets(name)]
    fn sets_name_a(self, value: String) -> String {
        value
    }

    #[sets(name)]
    fn sets_name_b(self, value: String) -> String {
        value
    }
}

fn main() {}
