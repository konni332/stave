#![allow(dead_code)]

use stave_macros::{builder, methods};

#[builder]
struct Cache<'a, T: Clone, const N: usize> {
    #[stave(required)]
    items: [T; N],
    #[stave(required)]
    name: &'a str,
    note: String,
}

#[methods]
impl Cache {
    #[sets(name)]
    fn set_name<S>(self, value: &'a S) -> &'a str
    where
        S: AsRef<str> + ?Sized,
    {
        value.as_ref()
    }

    #[sets(note)]
    #[requires(items, name)]
    fn set_note_from_self<D: std::fmt::Display>(mut self, prefix: D) -> String {
        format!(
            "{prefix}: {} items named {}",
            self.__stave_items.0.len(),
            self.__stave_name.0
        )
    }

    #[requires(items, name)]
    fn describe(&self) -> String {
        format!(
            "{} items named '{}'",
            self.__stave_items.0.len(),
            self.__stave_name.0
        )
    }
}

fn main() {
    let cache = Cache::new()
        .set_items([1, 2, 3])
        .set_name("numbers")
        .set_note_from_self("prefix");

    assert_eq!(cache.describe(), "3 items named 'numbers'");
    assert_eq!(cache.items(), &[1, 2, 3]);
    assert_eq!(cache.name(), &"numbers");
    assert_eq!(
        cache.note(),
        &Some("prefix: 3 items named numbers".to_string())
    );
}
