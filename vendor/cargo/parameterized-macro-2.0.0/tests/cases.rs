#[test]
fn individual_cases() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ok/01_import.rs");
    t.pass("tests/ok/02_multiple_ids.rs");
    t.pass("tests/ok/03_multiline.rs");
    t.pass("tests/ok/04_many_arg.rs");

    t.pass("tests/ok/06_vis.rs");
    t.pass("tests/ok/07_vis2.rs");
    t.pass("tests/ok/08_neg.rs");
    t.pass("tests/ok/09_option.rs");
    t.pass("tests/ok/10_result.rs");
    t.pass("tests/ok/11_enum.rs");
    t.pass("tests/ok/12_enum_with_variant_value.rs");
    t.pass("tests/ok/13_import_rename.rs");
    t.pass("tests/ok/14_transitive_attr.rs");
    t.pass("tests/ok/15_comment_in_test_case.rs");
    t.pass("tests/ok/16_trailing_comma1.rs");
    t.pass("tests/ok/17_trailing_comma2.rs");
    t.pass("tests/ok/18_trailing_comma3.rs");
    t.pass("tests/ok/19_trailing_comma4.rs");
    t.pass("tests/ok/20_empty.rs");
    t.pass("tests/ok/21_custom_test_attribute.rs");
    t.pass("tests/ok/22_custom_test_attribute_complex_meta.rs");

    t.compile_fail("tests/fail/id_already_defined.rs");
    t.compile_fail("tests/fail/inequal_amount_of_arg.rs");
    t.compile_fail("tests/fail/inequal_amount_of_arg_order.rs");
    t.compile_fail("tests/fail/input_param_order_in_err_message.rs");
    t.compile_fail("tests/fail/not_a_fn.rs");
    t.compile_fail("tests/fail/on_visibility.rs");
    t.compile_fail("tests/fail/multiple_custom_test_attributes.rs");

    #[cfg(not(feature = "square-brackets-old-error-message"))]
    t.compile_fail("tests/fail/square_brackets.rs");

    #[cfg(feature = "square-brackets-old-error-message")]
    t.compile_fail("tests/fail/square_brackets_old_error_message.rs");

    t.compile_fail("tests/fail/no_argument.rs");
    t.compile_fail("tests/fail/no_param.rs");
    t.compile_fail("tests/fail/no_param_nr2.rs");
}
