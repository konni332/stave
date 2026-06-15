use stave_macros::methods;

struct Widget;

#[methods]
impl Clone for Widget {
    fn clone(&self) -> Self {
        Widget
    }
}

fn main() {}
