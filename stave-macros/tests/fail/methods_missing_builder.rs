use stave_macros::methods;

struct Widget {
    name: String,
}

#[methods]
impl Widget {
    fn name(&self) -> &str {
        &self.name
    }
}

fn main() {}
