#![cfg(feature = "derive")]

#[test]
fn invalid_derive_inputs_fail_to_compile() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}

#[test]
fn valid_derive_inputs_compile() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/pass/*.rs");
}
