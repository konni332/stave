#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
mod fail {}
mod pass {
    mod builder {
        use stave_macros::builder;
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __ItemsUnset;
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __ItemsSet<T, const N: usize>([T; N]);
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __NameUnset;
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __NameSet<'a>(&'a str);
        struct Cache<'a, T: Clone, const N: usize, __ItemsState, __NameState> {
            __stave_items: __ItemsState,
            __stave_name: __NameState,
            note: ::core::option::Option<String>,
        }
        impl<'a, T: Clone, const N: usize> Cache<'a, T, N, __ItemsUnset, __NameUnset> {
            pub fn new() -> Self {
                Cache {
                    __stave_items: __ItemsUnset,
                    __stave_name: __NameUnset,
                    note: ::core::option::Option::None,
                }
            }
        }
    }
}
extern crate test;
#[rustc_test_marker = "derive_tests"]
#[doc(hidden)]
pub const derive_tests: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("derive_tests"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "stave-macros/tests/derive.rs",
        start_line: 5usize,
        start_col: 4usize,
        end_line: 5usize,
        end_col: 16usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(derive_tests()),
    ),
};
fn derive_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
    t.compile_fail("tests/fail/*.rs");
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&derive_tests])
}
