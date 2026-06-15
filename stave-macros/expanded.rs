#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
mod pass {
    mod builder {
        #![allow(dead_code)]
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
            __stave_phantom: ::core::marker::PhantomData<(&'a (), T, [(); N])>,
        }
        impl<'a, T: Clone, const N: usize> Cache<'a, T, N, __ItemsUnset, __NameUnset> {
            pub fn new() -> Self {
                Cache {
                    __stave_items: __ItemsUnset,
                    __stave_name: __NameUnset,
                    note: ::core::option::Option::None,
                    __stave_phantom: ::core::marker::PhantomData,
                }
            }
        }
        fn main() {}
    }
    mod methods {
        #![allow(dead_code)]
        use std::time::Duration;
        use stave_macros::{builder, methods};
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __HostUnset;
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __HostSet(String);
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __PortUnset;
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct __PortSet(u16);
        struct Server<__HostState, __PortState> {
            __stave_host: __HostState,
            __stave_port: __PortState,
            timeout: ::core::option::Option<Duration>,
            note: ::core::option::Option<String>,
        }
        impl Server<__HostUnset, __PortUnset> {
            pub fn new() -> Self {
                Server {
                    __stave_host: __HostUnset,
                    __stave_port: __PortUnset,
                    timeout: ::core::option::Option::None,
                    note: ::core::option::Option::None,
                }
            }
        }
        struct Config {
            host: String,
            port: u16,
            timeout: Option<Duration>,
            note: Option<String>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Config {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "Config",
                    "host",
                    &self.host,
                    "port",
                    &self.port,
                    "timeout",
                    &self.timeout,
                    "note",
                    &&self.note,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Config {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Config {
            #[inline]
            fn eq(&self, other: &Config) -> bool {
                self.port == other.port && self.host == other.host
                    && self.timeout == other.timeout && self.note == other.note
            }
        }
        impl<__PortState> Server<__HostUnset, __PortState> {
            fn sets_host(
                self,
                value: impl Into<String>,
            ) -> Server<__HostSet, __PortState> {
                let __stave_value: String = { value.into() };
                Server {
                    __stave_host: __HostSet(__stave_value),
                    __stave_port: self.__stave_port,
                    timeout: self.timeout,
                    note: self.note,
                }
            }
        }
        impl<__PortState> Server<__HostSet, __PortState> {
            fn sets_note_with_host(
                mut self,
                extra: &str,
            ) -> Server<__HostSet, __PortState> {
                let __stave_value: String = {
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("{0}-{1}", self.__stave_host.0, extra),
                        )
                    })
                };
                self.note = ::core::option::Option::Some(__stave_value);
                self
            }
        }
        impl Server<__HostSet, __PortSet> {
            fn finish(self) -> Config {
                Config {
                    host: self.__stave_host.0,
                    port: self.__stave_port.0,
                    timeout: self.timeout,
                    note: self.note,
                }
            }
        }
        impl<__PortState> Server<__HostSet, __PortState> {
            pub fn host(&self) -> &String {
                &self.__stave_host.0
            }
        }
        impl<__HostState> Server<__HostState, __PortUnset> {
            pub fn sets_port(self, value: u16) -> Server<__HostState, __PortSet> {
                Server {
                    __stave_host: self.__stave_host,
                    __stave_port: __PortSet(value),
                    timeout: self.timeout,
                    note: self.note,
                }
            }
        }
        impl<__HostState> Server<__HostState, __PortSet> {
            pub fn port(&self) -> &u16 {
                &self.__stave_port.0
            }
        }
        impl<__HostState, __PortState> Server<__HostState, __PortState> {
            pub fn sets_timeout(mut self, value: Duration) -> Self {
                self.timeout = ::core::option::Option::Some(value);
                self
            }
        }
        impl<__HostState, __PortState> Server<__HostState, __PortState> {
            pub fn timeout(&self) -> &::core::option::Option<Duration> {
                &self.timeout
            }
        }
        impl<__HostState, __PortState> Server<__HostState, __PortState> {
            pub fn note(&self) -> &::core::option::Option<String> {
                &self.note
            }
        }
        fn main() {
            let config1 = Server::new().sets_port(8080).sets_host("localhost").finish();
            let config2 = Server::new()
                .sets_host("localhost")
                .sets_port(8080)
                .sets_timeout(Duration::from_secs(5))
                .sets_note_with_host("extra")
                .finish();
            let host = config1.host();
            let port = config1.port();
            match (&host, &"localhost".to_string()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
            match (&port, &8080) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
            match (
                &config1,
                &Config {
                    host: "localhost".to_string(),
                    port: 8080,
                    timeout: None,
                    note: None,
                },
            ) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
            match (
                &config2,
                &Config {
                    host: "localhost".to_string(),
                    port: 8080,
                    timeout: Some(Duration::from_secs(5)),
                    note: Some("localhost-extra".to_string()),
                },
            ) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            }
        }
    }
    fn main() {}
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
        start_line: 4usize,
        start_col: 4usize,
        end_line: 4usize,
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
