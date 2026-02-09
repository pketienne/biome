mod language;
mod spec_test;

mod formatter {
    mod md_module {
        tests_macros::gen_tests! {"tests/specs/md/**/*.md", crate::spec_test::run, ""}
    }
}
