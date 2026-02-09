mod quick_test;
mod spec_test;

mod formatter {
    mod yaml_module {
        tests_macros::gen_tests! {"tests/specs/yaml/**/*.yaml", crate::spec_test::run, ""}
    }
}
