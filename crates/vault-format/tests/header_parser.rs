use base64::{Engine, engine::general_purpose::STANDARD};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use vault_format::{HeaderParseError, InputKind, VAULT_JSON_MAX_BYTES, parse_vault_header};

const VALID_KDF: &str = r#"{"algorithm":"Argon2id","memoryKiB":65536,"iterations":3,"parallelism":1,"salt":"AAAAAAAAAAAAAAAAAAAAAA=="}"#;
const VALID_ENVELOPE: &str = r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}"#;
const ROOT: [(&str, &str); 5] = [
    ("formatVersion", "1"),
    ("vaultId", r#""550e8400-e29b-41d4-a716-446655440000""#),
    ("createdAt", r#""2026-11-04T21:47:07Z""#),
    ("kdf", VALID_KDF),
    ("wrappedRegistryKey", VALID_ENVELOPE),
];

fn header(fields: &[(&str, &str)]) -> String {
    let members = fields
        .iter()
        .map(|(key, value)| format!(r#""{key}":{value}"#))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{members}}}")
}

fn valid_header() -> String {
    header(&ROOT)
}

fn header_without(field: &str) -> String {
    header(
        &ROOT
            .iter()
            .copied()
            .filter(|(name, _)| *name != field)
            .collect::<Vec<_>>(),
    )
}

fn replace_once(input: &str, from: &str, to: &str) -> String {
    assert!(
        input.contains(from),
        "fixture mutation target must exist: {from}"
    );
    input.replacen(from, to, 1)
}

fn parse_error(input: &[u8]) -> HeaderParseError {
    parse_vault_header(input).expect_err("input should be rejected")
}

fn b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

#[test]
fn parses_a_valid_header_as_explicitly_unauthenticated_metadata() {
    let input = valid_header();
    let parsed = parse_vault_header(input.as_bytes()).expect("synthetic header should parse");

    assert_eq!(parsed.format_version(), 1);
    assert_eq!(
        parsed.vault_id().to_string(),
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(parsed.kdf().memory_kib(), 65_536);
    assert_eq!(parsed.kdf().iterations(), 3);
    assert_eq!(parsed.kdf().parallelism(), 1);
    assert_eq!(parsed.kdf().salt_len(), 16);
    assert_eq!(parsed.wrapped_registry_key().version(), 1);
    assert_eq!(parsed.wrapped_registry_key().nonce_len(), 12);
    assert_eq!(parsed.wrapped_registry_key().ciphertext_len(), 48);
}

#[test]
fn permits_reordered_members_and_strict_unknown_root_values() {
    let input = header(&[
        (
            "future",
            r#"{"nested":[null,true,1.0,{"$serde_json::private::Number":"1"}]}"#,
        ),
        ("wrappedRegistryKey", VALID_ENVELOPE),
        ("createdAt", r#""2026-11-04T21:47:07Z""#),
        ("formatVersion", "1e0"),
        ("other", "null"),
        ("kdf", VALID_KDF),
        ("vaultId", r#""550e8400-e29b-41d4-a716-446655440000""#),
    ]);
    assert!(parse_vault_header(input.as_bytes()).is_ok());

    for value in ["null", "false", "0", r#""future text""#, "[]", "{}"] {
        let input = header_with_unknown(&format!(r#"{{"future":{value}}}"#));
        assert!(parse_vault_header(input.as_bytes()).is_ok(), "{value}");
    }
}

#[test]
fn missing_each_required_header_member_has_a_typed_error() {
    for name in [
        "formatVersion",
        "vaultId",
        "createdAt",
        "kdf",
        "wrappedRegistryKey",
    ] {
        assert_eq!(
            parse_error(header_without(name).as_bytes()),
            HeaderParseError::MissingField(name),
            "{name}"
        );
    }
}

#[test]
fn null_and_wrong_types_are_rejected_for_required_members() {
    for (name, value) in [
        ("formatVersion", "null"),
        ("vaultId", "null"),
        ("createdAt", "null"),
        ("kdf", "null"),
        ("wrappedRegistryKey", "null"),
        ("formatVersion", r#""1""#),
        ("vaultId", "17"),
        ("createdAt", "false"),
        ("kdf", "[]"),
        ("wrappedRegistryKey", "[]"),
    ] {
        let input = replace_once(
            &valid_header(),
            ROOT.iter().find(|(n, _)| *n == name).unwrap().1,
            value,
        );
        assert!(
            parse_vault_header(input.as_bytes()).is_err(),
            "{name}={value}"
        );
    }
    let wrong_kdf_type = replace_once(&valid_header(), VALID_KDF, "[]");
    assert_eq!(
        parse_error(wrong_kdf_type.as_bytes()),
        HeaderParseError::WrongFieldType("kdf")
    );
    let wrong_envelope_type = replace_once(&valid_header(), VALID_ENVELOPE, "[]");
    assert_eq!(
        parse_error(wrong_envelope_type.as_bytes()),
        HeaderParseError::WrongFieldType("wrappedRegistryKey")
    );
}

#[test]
fn rejects_non_object_root_after_strict_document_validation() {
    assert_eq!(parse_error(b"[]"), HeaderParseError::RootNotObject);
    assert_eq!(
        parse_error(b"[1, {\"x\":1,\"x\":2}]"),
        HeaderParseError::DuplicateProperty
    );
}

#[test]
fn rejects_duplicate_names_in_known_and_unknown_objects_after_escape_decoding() {
    for input in [
        r#"{"formatVersion":1,"formatVersion":1}"#,
        r#"{"vaultId":"a","vaultId":"b"}"#,
        r#"{"kdf":{},"kdf":{}}"#,
        r#"{"wrappedRegistryKey":{},"wrappedRegistryKey":{}}"#,
        r#"{"kdf":{"algorithm":"Argon2id","algorithm":"Argon2id"}}"#,
        r#"{"wrappedRegistryKey":{"version":1,"version":1}}"#,
        r#"{"future":1,"future":2}"#,
        r#"{"future":{"x":1,"x":2}}"#,
        r#"{"future":[{"x":1,"x":2}]}"#,
        r#"{"future":1,"\u0066uture":2}"#,
    ] {
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::DuplicateProperty,
            "{input}"
        );
    }
    let sibling_objects = r#"{"future":[{"x":1},{"x":2}]}"#;
    assert!(parse_vault_header(header_with_unknown(sibling_objects).as_bytes()).is_ok());
}

fn header_with_unknown(unknown_member: &str) -> String {
    let base = valid_header();
    let mut result = base[..base.len() - 1].to_owned();
    result.push(',');
    let member_body = unknown_member
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(unknown_member);
    result.push_str(member_body);
    result.push('}');
    result
}

#[test]
fn validates_complete_json_syntax_and_standard_whitespace() {
    assert!(parse_vault_header(format!(" \t{}\r\n", valid_header()).as_bytes()).is_ok());
    for bad in [
        "",
        "{}x",
        "{}{}",
        "{}\0",
        "{/*comment*/}",
        "{'formatVersion':1}",
        "{formatVersion:1}",
        r#"{"x":1,}"#,
        r#"{"x":"\uD800"}"#,
        r#"{"x":"\uDC00"}"#,
        r#"{"x":NaN}"#,
        r#"{"x":Infinity}"#,
    ] {
        assert_eq!(
            parse_error(bad.as_bytes()),
            HeaderParseError::InvalidJson,
            "{bad:?}"
        );
    }
    assert_eq!(parse_error(b"\xFF"), HeaderParseError::InvalidUtf8);
    assert_eq!(parse_error(b"\xEF\xBB\xBF{}"), HeaderParseError::InvalidBom);
    let mut truncated_utf8 = valid_header().into_bytes();
    truncated_utf8.extend_from_slice(&[0xC3]);
    assert_eq!(parse_error(&truncated_utf8), HeaderParseError::InvalidUtf8);
    let escaped_pair = header_with_unknown(r#"{"emoji":"\uD83D\uDE00"}"#);
    assert!(parse_vault_header(escaped_pair.as_bytes()).is_ok());
}

#[test]
fn rejects_depth_33_in_unknown_values_but_accepts_depth_31_and_32() {
    for nested_arrays in [30usize, 31] {
        let field = format!(
            r#""future":{}null{}"#,
            "[".repeat(nested_arrays),
            "]".repeat(nested_arrays)
        );
        assert!(
            parse_vault_header(header_with_unknown(&format!("{{{field}}}")).as_bytes()).is_ok()
        );
    }
    let depth_33 = format!(r#""future":{}null{}"#, "[".repeat(32), "]".repeat(32));
    assert_eq!(
        parse_error(header_with_unknown(&format!("{{{depth_33}}}")).as_bytes()),
        HeaderParseError::DepthLimitExceeded
    );

    let nested = format!("{}null{}", "{\"x\":".repeat(31), "}".repeat(31));
    let input = header_with_unknown(&format!(r#"{{"future":{nested}}}"#));
    assert!(parse_vault_header(input.as_bytes()).is_ok());
    let nested_33 = format!("{}null{}", "{\"x\":".repeat(32), "}".repeat(32));
    let input = header_with_unknown(&format!(r#"{{"future":{nested_33}}}"#));
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::DepthLimitExceeded
    );
}

#[test]
fn accepts_exact_integer_decimal_and_exponent_forms_for_version_and_kdf_fields() {
    for version in ["1", "1.0", "1e0", "10e-1"] {
        let input = replace_once(
            &valid_header(),
            r#""formatVersion":1"#,
            &format!(r#""formatVersion":{version}"#),
        );
        assert!(parse_vault_header(input.as_bytes()).is_ok(), "{version}");
    }
    let input = replace_once(
        &valid_header(),
        r#""memoryKiB":65536"#,
        r#""memoryKiB":655360e-1"#,
    );
    let input = replace_once(&input, r#""iterations":3"#, r#""iterations":30e-1"#);
    assert!(parse_vault_header(input.as_bytes()).is_ok());
}

#[test]
fn rejects_fractional_negative_overflow_and_extreme_exponents_without_rounding() {
    for (number, expected) in [
        ("1.5", HeaderParseError::NonIntegralNumber("formatVersion")),
        ("-1", HeaderParseError::NumericOutOfRange("formatVersion")),
        ("0", HeaderParseError::InvalidFormatVersion),
        (
            "18446744073709551616",
            HeaderParseError::NumericOutOfRange("formatVersion"),
        ),
        (
            "1e999999",
            HeaderParseError::NumericOutOfRange("formatVersion"),
        ),
        (
            "1e-999999",
            HeaderParseError::NonIntegralNumber("formatVersion"),
        ),
    ] {
        let input = replace_once(
            &valid_header(),
            r#""formatVersion":1"#,
            &format!(r#""formatVersion":{number}"#),
        );
        assert_eq!(parse_error(input.as_bytes()), expected, "{number}");
    }
    let input = replace_once(
        &valid_header(),
        r#""memoryKiB":65536"#,
        r#""memoryKiB":1.5"#,
    );
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::NonIntegralNumber("memoryKiB")
    );

    let long_token = "9".repeat(200_000);
    let input = replace_once(
        &valid_header(),
        r#""formatVersion":1"#,
        &format!(r#""formatVersion":{long_token}"#),
    );
    assert!(input.len() < VAULT_JSON_MAX_BYTES);
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::NumericOutOfRange("formatVersion")
    );
}

#[test]
fn rejects_a_numeric_field_that_mimics_the_arbitrary_precision_private_object() {
    let input = replace_once(
        &valid_header(),
        r#""formatVersion":1"#,
        r#""formatVersion":{"$serde_json::private::Number":"1"}"#,
    );
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::WrongFieldType("formatVersion")
    );
    let accepted_unknown =
        header_with_unknown(r#"{"future":{"$serde_json::private::Number":"1"}}"#);
    assert!(parse_vault_header(accepted_unknown.as_bytes()).is_ok());
}

#[test]
fn classifies_future_versions_only_after_the_whole_document_passes_strict_json() {
    let input = replace_once(
        &valid_header(),
        r#""formatVersion":1"#,
        r#""formatVersion":2.0"#,
    );
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::UnsupportedVersion(2)
    );
    let malformed = format!("{input} trailing");
    assert_eq!(
        parse_error(malformed.as_bytes()),
        HeaderParseError::InvalidJson
    );
    let duplicate_unknown = replace_once(&input, "}", r#","future":1,"future":2}"#);
    assert_eq!(
        parse_error(duplicate_unknown.as_bytes()),
        HeaderParseError::DuplicateProperty
    );
}

#[test]
fn validates_canonical_lowercase_uuid_text_without_requiring_uuid_v4() {
    let valid_non_v4 = replace_once(
        &valid_header(),
        "550e8400-e29b-41d4-a716-446655440000",
        "00000000-0000-0000-0000-000000000000",
    );
    assert!(parse_vault_header(valid_non_v4.as_bytes()).is_ok());
    for bad_id in [
        "550E8400-E29B-41D4-A716-446655440000",
        "550e8400e29b41d4a716446655440000",
        "{550e8400-e29b-41d4-a716-446655440000}",
        "urn:uuid:550e8400-e29b-41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-44665544000g",
    ] {
        let input = replace_once(
            &valid_header(),
            "550e8400-e29b-41d4-a716-446655440000",
            bad_id,
        );
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::InvalidIdentifier,
            "{bad_id}"
        );
    }
}

#[test]
fn accepts_canonical_utc_timestamps_and_rejects_normalized_alternatives() {
    for timestamp in [
        "2026-11-04T21:47:07Z",
        "2026-11-04T21:47:07.1Z",
        "2026-11-04T21:47:07.123456789Z",
    ] {
        let input = replace_once(&valid_header(), "2026-11-04T21:47:07Z", timestamp);
        assert!(parse_vault_header(input.as_bytes()).is_ok(), "{timestamp}");
    }
    for timestamp in [
        "2026-11-04T21:47:07+00:00",
        "2026-11-04t21:47:07z",
        "2026-11-04T23:59:60Z",
        "2026-11-04T24:00:00Z",
        "2026-02-30T00:00:00Z",
        "2026-11-04T21:47Z",
        "2026-11-04T21:47:07.1234567890Z",
        "2026-11-04T21:47:07",
    ] {
        let input = replace_once(&valid_header(), "2026-11-04T21:47:07Z", timestamp);
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::InvalidTimestamp,
            "{timestamp}"
        );
    }
}

#[test]
fn rejects_leap_seconds_before_time_library_normalization() {
    for timestamp in [
        "2016-12-31T23:59:60Z",
        "2016-12-31T23:59:60.5Z",
        "2026-11-30T23:59:60Z",
        "2026-11-30T23:59:60.5Z",
        "2026-06-30T23:59:60Z",
        "2026-11-04T23:59:60Z",
        "2026-11-30T23:59:61Z",
        "2026-11-30T23:59:99Z",
    ] {
        let input = replace_once(&valid_header(), "2026-11-04T21:47:07Z", timestamp);
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::InvalidTimestamp,
            "{timestamp}"
        );
    }
}

#[test]
fn accepts_the_valid_second_before_minute_end_and_month_end() {
    for timestamp in [
        "2016-12-31T23:59:59Z",
        "2016-12-31T23:59:59.999999999Z",
        "2026-11-30T23:59:59Z",
    ] {
        let input = replace_once(&valid_header(), "2026-11-04T21:47:07Z", timestamp);
        assert!(parse_vault_header(input.as_bytes()).is_ok(), "{timestamp}");
    }
}

#[test]
fn validates_kdf_ranges_with_checked_boundaries() {
    for (parallelism, memory, iterations, valid) in [
        (1, 7, 1, false),
        (1, 8, 1, true),
        (1, 9, 1, true),
        (8, 63, 1, false),
        (8, 64, 1, true),
        (8, 65, 1, true),
        (1, 1_048_575, 1, true),
        (1, 1_048_576, 10, true),
        (1, 1_048_577, 1, false),
        (0, 8, 1, false),
        (9, 72, 1, false),
        (1, 8, 0, false),
        (1, 8, 11, false),
    ] {
        let mut input = valid_header();
        input = replace_once(
            &input,
            r#""memoryKiB":65536"#,
            &format!(r#""memoryKiB":{memory}"#),
        );
        input = replace_once(
            &input,
            r#""iterations":3"#,
            &format!(r#""iterations":{iterations}"#),
        );
        input = replace_once(
            &input,
            r#""parallelism":1"#,
            &format!(r#""parallelism":{parallelism}"#),
        );
        if valid {
            assert!(
                parse_vault_header(input.as_bytes()).is_ok(),
                "{parallelism}/{memory}/{iterations}"
            );
        } else {
            assert!(
                parse_vault_header(input.as_bytes()).is_err(),
                "{parallelism}/{memory}/{iterations}"
            );
        }
    }
}

#[test]
fn validates_stored_kdf_algorithm_and_rejects_unknown_fields() {
    let bad_algorithm = replace_once(&valid_header(), "Argon2id", "Argon2i");
    assert_eq!(
        parse_error(bad_algorithm.as_bytes()),
        HeaderParseError::InvalidKdfParameters("algorithm")
    );
    let extra = replace_once(
        &valid_header(),
        r#""parallelism":1"#,
        r#""parallelism":1,"extra":null"#,
    );
    assert_eq!(
        parse_error(extra.as_bytes()),
        HeaderParseError::ForbiddenField
    );
    let reordered = replace_once(
        VALID_KDF,
        r#""algorithm":"Argon2id","memoryKiB":65536,"iterations":3,"parallelism":1"#,
        r#""parallelism":1,"iterations":3,"memoryKiB":65536,"algorithm":"Argon2id""#,
    );
    let input = replace_once(&valid_header(), VALID_KDF, &reordered);
    assert!(parse_vault_header(input.as_bytes()).is_ok());
}

#[test]
fn every_kdf_and_envelope_member_is_required_individually() {
    for (field, needle) in [
        ("algorithm", r#""algorithm":"Argon2id","#),
        ("memoryKiB", r#""memoryKiB":65536,"#),
        ("iterations", r#""iterations":3,"#),
        ("parallelism", r#""parallelism":1,"#),
        ("salt", r#","salt":"AAAAAAAAAAAAAAAAAAAAAA==""#),
    ] {
        let kdf = replace_once(VALID_KDF, needle, "");
        let input = replace_once(&valid_header(), VALID_KDF, &kdf);
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::MissingField(field)
        );
    }

    for (field, needle) in [
        ("version", r#""version":1,"#),
        ("algorithm", r#""algorithm":"AES/GCM/NoPadding","#),
        ("nonce", r#""nonce":"AAAAAAAAAAAAAAAA","#),
        (
            "ciphertext",
            r#","ciphertext":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""#,
        ),
    ] {
        let envelope = replace_once(VALID_ENVELOPE, needle, "");
        let input = replace_once(&valid_header(), VALID_ENVELOPE, &envelope);
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::MissingField(field)
        );
    }
}

#[test]
fn nested_required_values_reject_null_and_incorrect_json_types() {
    for (needle, replacement, field) in [
        (
            r#""algorithm":"Argon2id""#,
            r#""algorithm":null"#,
            "algorithm",
        ),
        (
            r#""memoryKiB":65536"#,
            r#""memoryKiB":"65536""#,
            "memoryKiB",
        ),
        (r#""iterations":3"#, r#""iterations":false"#, "iterations"),
        (r#""parallelism":1"#, r#""parallelism":null"#, "parallelism"),
        (
            r#""salt":"AAAAAAAAAAAAAAAAAAAAAA==""#,
            r#""salt":17"#,
            "salt",
        ),
        (r#""version":1"#, r#""version":null"#, "version"),
        (
            r#""algorithm":"AES/GCM/NoPadding""#,
            r#""algorithm":true"#,
            "algorithm",
        ),
        (r#""nonce":"AAAAAAAAAAAAAAAA""#, r#""nonce":null"#, "nonce"),
        (
            r#""ciphertext":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""#,
            r#""ciphertext":[]"#,
            "ciphertext",
        ),
    ] {
        let input = replace_once(&valid_header(), needle, replacement);
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::WrongFieldType(field),
            "{field}"
        );
    }
}

#[test]
fn salt_requires_canonical_standard_padded_base64_and_exactly_16_bytes() {
    for salt in [
        b64(&[0; 15]),
        b64(&[0; 17]),
        "not-base64!".to_owned(),
        b64(&[0xFF; 16]).replacen('/', "_", 1),
        "AAAAAAAAAAAAAAAAAAAAAA".to_owned(),
        "AAAAAAAAAAAAAAAAAAAAAA=A".to_owned(),
        "AAAAAAAAAAAAAAAAAAAAAB==".to_owned(),
        "AAAAAAAAAAAAAAAAAAAAAA===".to_owned(),
        "AAAAAAAAAAAAAAAAAAAA AA==".to_owned(),
    ] {
        let input = replace_once(&valid_header(), "AAAAAAAAAAAAAAAAAAAAAA==", &salt);
        assert!(parse_vault_header(input.as_bytes()).is_err(), "{salt}");
    }
    let input = replace_once(&valid_header(), "AAAAAAAAAAAAAAAAAAAAAA==", &b64(&[0; 16]));
    assert!(parse_vault_header(input.as_bytes()).is_ok());
}

#[test]
fn fixed_base64_fields_decode_json_escapes_without_changing_base64_canonicality() {
    let escaped_ascii = replace_once(&valid_header(), r#""salt":"A"#, r#""salt":"\u0041"#);
    let parsed = parse_vault_header(escaped_ascii.as_bytes()).expect("escaped A remains A");
    assert_eq!(parsed.kdf().salt(), &[0; 16]);

    let salt = b64(&[0xFF; 16]);
    let input = replace_once(&valid_header(), "AAAAAAAAAAAAAAAAAAAAAA==", &salt);
    let escaped_slash = salt.replacen('/', r#"\/"#, 1);
    let input = replace_once(&input, &salt, &escaped_slash);
    let parsed = parse_vault_header(input.as_bytes()).expect("escaped slash remains slash");
    assert_eq!(parsed.kdf().salt(), &[0xFF; 16]);
}

#[test]
fn oversized_base64_text_is_rejected_without_becoming_an_unbounded_decoded_value() {
    let oversized = format!("\"salt\":\"{}\"", "A".repeat(VAULT_JSON_MAX_BYTES - 1024));
    let input = replace_once(
        &valid_header(),
        r#""salt":"AAAAAAAAAAAAAAAAAAAAAA==""#,
        &oversized,
    );
    assert!(input.len() < VAULT_JSON_MAX_BYTES);
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::InvalidBase64("salt")
    );
}

#[test]
fn nested_envelope_requires_exact_schema_version_algorithm_and_base64_lengths() {
    let extra = replace_once(
        &valid_header(),
        r#""version":1,"algorithm""#,
        r#""version":1,"extra":null,"algorithm""#,
    );
    assert_eq!(
        parse_error(extra.as_bytes()),
        HeaderParseError::ForbiddenField
    );
    let version = replace_once(&valid_header(), r#""version":1"#, r#""version":2"#);
    assert_eq!(
        parse_error(version.as_bytes()),
        HeaderParseError::InvalidEnvelope("version")
    );
    let algorithm = replace_once(&valid_header(), "AES/GCM/NoPadding", "AES/GCM");
    assert_eq!(
        parse_error(algorithm.as_bytes()),
        HeaderParseError::InvalidEnvelope("algorithm")
    );
    let missing = replace_once(
        &valid_header(),
        r#","ciphertext":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""#,
        "",
    );
    assert_eq!(
        parse_error(missing.as_bytes()),
        HeaderParseError::MissingField("ciphertext")
    );
    let reordered = replace_once(
        &valid_header(),
        VALID_ENVELOPE,
        r#"{"ciphertext":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA","nonce":"AAAAAAAAAAAAAAAA","algorithm":"AES/GCM/NoPadding","version":1}"#,
    );
    assert!(parse_vault_header(reordered.as_bytes()).is_ok());
}

#[test]
fn envelope_nonce_and_ciphertext_lengths_are_checked_without_unbounded_allocation() {
    for length in [11usize, 13] {
        let input = replace_once(
            &valid_header(),
            r#""nonce":"AAAAAAAAAAAAAAAA""#,
            &format!(r#""nonce":"{}""#, b64(&vec![0; length])),
        );
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::InvalidBase64("nonce")
        );
    }
    for length in [47usize, 49] {
        let input = replace_once(
            &valid_header(),
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            &b64(&vec![0; length]),
        );
        assert_eq!(
            parse_error(input.as_bytes()),
            HeaderParseError::InvalidBase64("ciphertext")
        );
    }
    let bad_nonce = replace_once(
        &valid_header(),
        r#""nonce":"AAAAAAAAAAAAAAAA""#,
        r#""nonce":"AAAAAAAAAAAAAA==""#,
    );
    assert_eq!(
        parse_error(bad_nonce.as_bytes()),
        HeaderParseError::InvalidBase64("nonce")
    );
    let unpadded_nonce = replace_once(
        &valid_header(),
        r#""nonce":"AAAAAAAAAAAAAAAA""#,
        r#""nonce":"AAAAAAAAAAAAAA""#,
    );
    assert_eq!(
        parse_error(unpadded_nonce.as_bytes()),
        HeaderParseError::InvalidBase64("nonce")
    );
    let invalid_ciphertext = replace_once(
        &valid_header(),
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA!",
    );
    assert_eq!(
        parse_error(invalid_ciphertext.as_bytes()),
        HeaderParseError::InvalidBase64("ciphertext")
    );
}

#[test]
fn input_limit_is_inclusive_and_checked_before_any_json_work() {
    let valid = valid_header();
    let prefix = &valid[..valid.len() - 1];
    let empty_padding = format!("{prefix},\"padding\":\"\"}}");
    let exact = format!(
        "{prefix},\"padding\":\"{}\"}}",
        "a".repeat(VAULT_JSON_MAX_BYTES - empty_padding.len())
    );
    assert_eq!(exact.len(), VAULT_JSON_MAX_BYTES);
    assert!(parse_vault_header(exact.as_bytes()).is_ok());
    let too_long = vec![0xFF; VAULT_JSON_MAX_BYTES + 1];
    assert_eq!(
        parse_error(&too_long),
        HeaderParseError::InputTooLarge {
            actual_bytes: VAULT_JSON_MAX_BYTES + 1
        }
    );
    assert_eq!(InputKind::VaultHeader.max_bytes(), VAULT_JSON_MAX_BYTES);
}

#[test]
fn unknown_fields_have_no_artificial_member_count_limit_within_the_file_ceiling() {
    let members = (0..1_000)
        .map(|n| format!(r#""future{n}":{{"x{n}":null}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let input = header_with_unknown(&format!("{{{members}}}"));
    assert!(input.len() < VAULT_JSON_MAX_BYTES);
    assert!(parse_vault_header(input.as_bytes()).is_ok());
}

#[test]
fn repeated_malformed_unknown_content_remains_bounded_and_detects_the_final_duplicate() {
    let members = (0..12_000)
        .map(|n| format!(r#""future{n}":null"#))
        .collect::<Vec<_>>()
        .join(",");
    let unknown = format!(r#"{{{members},"future0":false}}"#);
    let input = header_with_unknown(&unknown);
    assert!(input.len() < VAULT_JSON_MAX_BYTES);
    assert_eq!(
        parse_error(input.as_bytes()),
        HeaderParseError::DuplicateProperty
    );
}

#[test]
fn diagnostics_and_debug_never_echo_untrusted_json_base64_or_ciphertext() {
    let duplicate =
        r#"{"sentinel-property-name":"sentinel-secret-value","sentinel-property-name":"again"}"#;
    let error = parse_error(duplicate.as_bytes());
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("sentinel"));
    let bad_salt = replace_once(
        &valid_header(),
        "AAAAAAAAAAAAAAAAAAAAAA==",
        "DO-NOT-LEAK-THIS",
    );
    let error = parse_error(bad_salt.as_bytes());
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("DO-NOT-LEAK"));

    let bad_uuid = replace_once(
        &valid_header(),
        "550e8400-e29b-41d4-a716-446655440000",
        "UUID-SENTINEL-DO-NOT-LEAK-0000000000000",
    );
    let error = parse_error(bad_uuid.as_bytes());
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("UUID-SENTINEL"));

    let malformed_json = r#"{"untrusted":"JSON-SENTINEL-DO-NOT-LEAK\q"}"#;
    let error = parse_error(malformed_json.as_bytes());
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("JSON-SENTINEL"));

    let bad_base64 = replace_once(
        &valid_header(),
        "AAAAAAAAAAAAAAAAAAAAAA==",
        "BASE64-SENTINEL-DO-NOT-LEAK",
    );
    let error = parse_error(bad_base64.as_bytes());
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("BASE64-SENTINEL"));

    let parsed = parse_vault_header(valid_header().as_bytes()).unwrap();
    let rendered = format!("{parsed:?}");
    assert!(!rendered.contains("AAAAAAAA"));
}

#[test]
fn debug_output_redacts_vault_identifier_timestamp_and_cryptographic_bytes() {
    let vault_id = "deadbeef-cafe-4bad-8dad-0123456789ab";
    let timestamp = "2026-10-29T13:14:15.123456789Z";
    let salt: [u8; 16] = std::array::from_fn(|index| 0xA0 + index as u8);
    let nonce: [u8; 12] = std::array::from_fn(|index| 0x40 + index as u8);
    let ciphertext: [u8; 48] = std::array::from_fn(|index| 0x80 + index as u8);

    let mut input = replace_once(
        &valid_header(),
        "550e8400-e29b-41d4-a716-446655440000",
        vault_id,
    );
    input = replace_once(&input, "2026-11-04T21:47:07Z", timestamp);
    input = replace_once(&input, "AAAAAAAAAAAAAAAAAAAAAA==", &b64(&salt));
    input = replace_once(&input, "AAAAAAAAAAAAAAAA", &b64(&nonce));
    input = replace_once(
        &input,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        &b64(&ciphertext),
    );

    let parsed = parse_vault_header(input.as_bytes()).expect("synthetic header should parse");
    let parsed_debug = format!("{parsed:?}");
    let vault_id_debug = format!("{:?}", parsed.vault_id());
    let vault_id_bytes_debug = format!("{:?}", parsed.vault_id().as_bytes());
    let kdf_debug = format!("{:?}", parsed.kdf());
    let envelope_debug = format!("{:?}", parsed.wrapped_registry_key());

    for rendered in [&parsed_debug, &vault_id_debug, &kdf_debug, &envelope_debug] {
        assert!(
            !rendered.contains(vault_id),
            "debug leaked the UUID: {rendered}"
        );
        assert!(
            !rendered.contains("deadbeef") && !rendered.contains("0123456789ab"),
            "debug leaked recognizable UUID text: {rendered}"
        );
        assert!(
            !rendered.contains(&vault_id_bytes_debug),
            "debug leaked raw UUID bytes: {rendered}"
        );
        assert!(
            !rendered.contains("2026"),
            "debug leaked the timestamp: {rendered}"
        );
        assert!(
            !rendered.contains(&b64(&salt))
                && !rendered.contains(&b64(&nonce))
                && !rendered.contains(&b64(&ciphertext)),
            "debug leaked encoded cryptographic bytes: {rendered}"
        );
        assert!(
            !rendered.contains(&format!("{salt:?}"))
                && !rendered.contains(&format!("{nonce:?}"))
                && !rendered.contains(&format!("{ciphertext:?}")),
            "debug leaked raw cryptographic bytes: {rendered}"
        );
    }

    assert!(parsed_debug.contains("ParsedUnauthenticatedVaultHeader"));
    assert!(parsed_debug.contains("format_version: 1"));
    assert!(parsed_debug.contains("salt_bytes: 16"));
    assert!(parsed_debug.contains("nonce_bytes: 12"));
    assert!(parsed_debug.contains("ciphertext_bytes: 48"));
    assert!(vault_id_debug.contains("VaultId"));
    assert_eq!(parsed.vault_id().to_string(), vault_id);
    assert_eq!(
        parsed.vault_id().as_bytes(),
        &[
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0x4B, 0xAD, 0x8D, 0xAD, 0x01, 0x23, 0x45, 0x67,
            0x89, 0xAB
        ]
    );
    assert_eq!(
        parsed.created_at().unix_timestamp_nanos(),
        OffsetDateTime::parse(timestamp, &Rfc3339)
            .expect("test timestamp should parse")
            .unix_timestamp_nanos()
    );
}

#[test]
fn malformed_inputs_are_panic_free_and_error_categories_are_deterministic() {
    for bytes in [
        b"".as_slice(),
        b"{".as_slice(),
        b"[]".as_slice(),
        b"\xFF".as_slice(),
        b"{\"a\":1,\"a\":2}".as_slice(),
        b"{\"createdAt\":\"2026-02-30T00:00:00Z\"}".as_slice(),
    ] {
        let first = std::panic::catch_unwind(|| parse_vault_header(bytes));
        let second = std::panic::catch_unwind(|| parse_vault_header(bytes));
        assert!(first.is_ok() && second.is_ok());
        assert_eq!(first.unwrap().unwrap_err(), second.unwrap().unwrap_err());
    }
}
