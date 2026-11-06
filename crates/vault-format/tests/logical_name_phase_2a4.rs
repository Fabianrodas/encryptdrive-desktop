use vault_format::{
    LogicalNameError, logical_name_eq_ignore_case, strip_java_whitespace, validate_logical_name,
};

#[test]
fn name_01_uses_code_point_lengths_after_stripping() {
    assert_eq!(validate_logical_name(""), Err(LogicalNameError::Empty));
    assert_eq!(validate_logical_name("a").unwrap(), "a");
    assert_eq!(
        validate_logical_name(&"a".repeat(254))
            .unwrap()
            .chars()
            .count(),
        254
    );
    assert_eq!(
        validate_logical_name(&"a".repeat(255))
            .unwrap()
            .chars()
            .count(),
        255
    );
    assert_eq!(
        validate_logical_name(&"a".repeat(256)),
        Err(LogicalNameError::TooLong)
    );
    assert_eq!(
        validate_logical_name(&format!("{}🦀", "a".repeat(254)))
            .unwrap()
            .chars()
            .count(),
        255
    );
    assert_eq!(
        validate_logical_name(&format!("{}🦀", "a".repeat(255))),
        Err(LogicalNameError::TooLong)
    );
    assert_eq!(validate_logical_name("  a \u{1c}").unwrap(), "a");
}

#[test]
fn name_02_strips_exact_java_whitespace_and_preserves_nonbreaking_spaces() {
    assert_eq!(
        strip_java_whitespace("\u{1c}\u{20}Name\u{20}\u{1f}"),
        "Name"
    );
    assert_eq!(
        strip_java_whitespace("\u{a0}Name\u{a0}"),
        "\u{a0}Name\u{a0}"
    );
    assert_eq!(
        strip_java_whitespace("\u{2007}Name\u{202f}"),
        "\u{2007}Name\u{202f}"
    );
    assert_eq!(
        strip_java_whitespace("\u{85}Name\u{85}"),
        "\u{85}Name\u{85}"
    );
    assert_eq!(strip_java_whitespace("\u{2000}Name\u{200a}"), "Name");
    assert_eq!(
        strip_java_whitespace(" \u{a0} \u{1c}Name\u{1f} \u{a0} "),
        "\u{a0} \u{1c}Name\u{1f} \u{a0}"
    );
    assert_eq!(strip_java_whitespace(" \u{2000}\u{1c}"), "");
}

#[test]
fn name_03_rejects_only_the_approved_empty_dot_dotdot_and_nul_cases() {
    for value in ["", " \u{1c}\u{2000} "] {
        assert_eq!(validate_logical_name(value), Err(LogicalNameError::Empty));
    }
    assert_eq!(validate_logical_name("."), Err(LogicalNameError::Dot));
    assert_eq!(validate_logical_name(".."), Err(LogicalNameError::DotDot));
    assert_eq!(
        validate_logical_name("a\0b"),
        Err(LogicalNameError::ContainsNul)
    );
}

#[test]
fn name_04_preserves_accepted_characters_including_slashes_and_windows_names() {
    for value in [
        "\u{1}control",
        "inside\ttab",
        "/",
        "folder/name",
        "folder\\name",
        "name:stream",
        "CON",
        "NUL.txt",
        "trailing.",
    ] {
        assert_eq!(validate_logical_name(value).unwrap(), value, "{value:?}");
    }
}

#[test]
fn name_05_does_not_normalize_and_counts_combining_marks_as_code_points() {
    let composed = "caf\u{e9}";
    let decomposed = "cafe\u{301}";
    assert_eq!(validate_logical_name(composed).unwrap(), composed);
    assert_eq!(validate_logical_name(decomposed).unwrap(), decomposed);
    assert_ne!(composed, decomposed);
    assert!(!logical_name_eq_ignore_case(composed, decomposed));
    assert_eq!("a\u{301}".chars().count(), 2);
    assert_eq!(validate_logical_name("🦀").unwrap(), "🦀");
}

#[test]
fn name_06_matches_java_equals_ignore_case_cases() {
    for (left, right) in [
        ("Photos", "PHOTOS"),
        ("name", "NAME"),
        ("I", "ı"),
        ("İ", "i"),
        ("Σ", "ς"),
        ("Σ", "σ"),
        ("K", "K"),
        ("𐐀", "𐐨"),
        ("𐐀A", "𐐨a"),
    ] {
        assert!(
            logical_name_eq_ignore_case(left, right),
            "{left:?} / {right:?}"
        );
    }
    for (left, right) in [
        ("ß", "SS"),
        ("é", "e\u{301}"),
        ("İ", "i\u{307}"),
        ("Photos", "PHOTOS!"),
    ] {
        assert!(
            !logical_name_eq_ignore_case(left, right),
            "{left:?} / {right:?}"
        );
    }
    assert!(logical_name_eq_ignore_case("Photos", "PHOTOS")); // parent is intentionally absent
}

#[test]
fn logical_name_errors_are_sanitized() {
    let sentinel = "private sentinel name";
    let value = format!("{sentinel}{}", "x".repeat(256));
    let error = validate_logical_name(&value).unwrap_err();
    assert!(!format!("{error}").contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));
}
