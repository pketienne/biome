mod quick_test;
mod spec_test;

mod formatter {
    mod turtle_module {
        tests_macros::gen_tests! {"tests/specs/turtle/**/*.ttl", crate::spec_test::run, ""}
    }
}
