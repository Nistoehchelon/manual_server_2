use manual_server::storage::{
    FIELD_SEPARATOR, FILE_ENDING_MARKER, RECORD_SEPARATOR, escape, unescape,
};

#[test]
fn escapes_and_unescapes_how_i_expect_it_to() {
    // I guess I will edit this if I change them, and these asserts will help me?
    assert_eq!(
        FILE_ENDING_MARKER, "{]]]}",
        "FILE_ENDING_MARKER has been modified since this test was last written. Go into tests and update the assertions"
    );
    assert_eq!(
        FIELD_SEPARATOR, "{|||}",
        "FIELD_SEPARATOR has been modified since this test was last written. Go into tests and update the assertions"
    );
    assert_eq!(
        RECORD_SEPARATOR, "{>>>}",
        "RECORD_SEPARATOR has been modified since this test was last written. Go into tests and update the assertions"
    );

    let string_normal = String::from(r"{>>>} {]]]} {]]");
    let string_escaped = String::from(r"{\>>>\} {\]]]\} {]]");
    let escaped = escape(string_normal.clone());
    assert_eq!(escaped, string_escaped);
    assert_eq!(unescape(escaped), string_normal);
}
