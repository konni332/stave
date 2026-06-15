use stave_macros::methods;

struct Widget;

#[methods]
impl<T> Widget {
    fn noop(&self) {}
}

fn main() {}
