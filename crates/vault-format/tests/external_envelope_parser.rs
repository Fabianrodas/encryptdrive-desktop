use base64::{Engine, engine::general_purpose::STANDARD};
use vault_format::{
    ExternalEnvelopeField, ExternalEnvelopeParseError, InputKind, UnauthenticatedExternalEnvelope,
    parse_encrypted_account_manifest, parse_encrypted_user_registry,
};

const NONCE: [u8; 12] = [0; 12];
const ALGORITHM: &str = "AES/GCM/NoPadding";

fn b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

fn envelope(version: &str, algorithm: &str, nonce: &str, ciphertext: &str) -> String {
    format!(
        r#"{{"version":{version},"algorithm":"{algorithm}","nonce":"{nonce}","ciphertext":"{ciphertext}"}}"#
    )
}

fn valid_envelope_with_len(ciphertext_len: usize) -> String {
    let ciphertext = vec![0_u8; ciphertext_len];
    envelope("1", ALGORITHM, &b64(&NONCE), &b64(&ciphertext))
}

fn user_error(input: &[u8]) -> ExternalEnvelopeParseError {
    parse_encrypted_user_registry(input).expect_err("synthetic input must be rejected")
}

fn replace_once(input: &str, from: &str, to: &str) -> String {
    assert!(input.contains(from), "fixture mutation target must exist");
    input.replacen(from, to, 1)
}

fn maximum_size_envelope(maximum: usize, trailing_whitespace: usize) -> Vec<u8> {
    const PREFIX: &[u8] = b"{\"version\":1,\"algorithm\":\"AES/GCM/NoPadding\",\"nonce\":\"AAAAAAAAAAAAAAAA\",\"ciphertext\":\"";
    const SUFFIX: &[u8] = b"\"}";
    let overhead = PREFIX.len() + SUFFIX.len();
    assert!(maximum >= overhead + trailing_whitespace);
    let encoded_len = maximum - overhead - trailing_whitespace;
    assert_eq!(encoded_len % 4, 0, "fixture uses whole Base64 quartets");

    let mut input = Vec::with_capacity(maximum);
    input.extend_from_slice(PREFIX);
    input.resize(input.len() + encoded_len, b'A');
    input.extend_from_slice(SUFFIX);
    input.resize(input.len() + trailing_whitespace, b' ');
    assert_eq!(input.len(), maximum);
    input
}

fn assert_file_boundaries(
    maximum: usize,
    kind: InputKind,
    expected_below: usize,
    expected_exact: usize,
    parser: fn(&[u8]) -> Result<UnauthenticatedExternalEnvelope, ExternalEnvelopeParseError>,
) {
    for (length, trailing_whitespace, expected_ciphertext_len) in [
        (maximum - 1, 3, expected_below),
        (maximum, 0, expected_exact),
    ] {
        let input = maximum_size_envelope(length, trailing_whitespace);
        let parsed = parser(&input).expect("valid at-limit envelope should parse");
        assert_eq!(parsed.version(), 1);
        assert_eq!(parsed.ciphertext_len(), expected_ciphertext_len);
        drop(parsed);
        drop(input);
    }

    let oversized = vec![b' '; maximum + 1];
    assert_eq!(
        parser(&oversized).unwrap_err(),
        ExternalEnvelopeParseError::InputTooLarge(kind)
    );
}

#[test]
fn a01_a08_accepts_both_categories_and_variable_synthetic_ciphertexts() {
    for length in [16, 17, 48, 257] {
        let input = valid_envelope_with_len(length);
        let registry = parse_encrypted_user_registry(input.as_bytes())
            .expect("synthetic registry envelope should be structurally valid");
        let manifest = parse_encrypted_account_manifest(input.as_bytes())
            .expect("synthetic manifest envelope should be structurally valid");
        assert_eq!(registry.version(), 1);
        assert_eq!(registry.nonce(), &NONCE);
        assert_eq!(registry.ciphertext_and_tag().len(), length);
        assert_eq!(manifest.ciphertext_and_tag().len(), length);
    }
}

#[test]
fn a03_a04_b17_accepts_reordered_fields_whitespace_escapes_and_exact_numeric_one() {
    let ciphertext = b64(&[0; 16]);
    let input = format!(
        " \r\n{{\"ciphertext\":\"{ciphertext}\",\"\\u006eonce\":\"AAAAAAAAAAAAAAAA\",\"algorithm\":\"AES/GCM/NoPadding\",\"version\":1.00e+0}}\t\n"
    );

    let parsed = parse_encrypted_user_registry(input.as_bytes())
        .expect("valid escaped spellings and exact numeric one should parse");
    assert_eq!(parsed.version(), 1);
    assert_eq!(parsed.ciphertext_and_tag().len(), 16);

    for version in ["1", "1.0", "1e0", "1.00e+0"] {
        let input =
            valid_envelope_with_len(16).replace("\"version\":1", &format!("\"version\":{version}"));
        assert!(
            parse_encrypted_user_registry(input.as_bytes()).is_ok(),
            "{version}"
        );
    }
}

#[test]
fn b01_b16_rejects_schema_and_value_violations_with_typed_errors() {
    let valid = valid_envelope_with_len(16);
    for (field, member) in [
        (ExternalEnvelopeField::Version, "\"version\":1,"),
        (
            ExternalEnvelopeField::Algorithm,
            ",\"algorithm\":\"AES/GCM/NoPadding\"",
        ),
        (
            ExternalEnvelopeField::Nonce,
            ",\"nonce\":\"AAAAAAAAAAAAAAAA\"",
        ),
        (
            ExternalEnvelopeField::Ciphertext,
            ",\"ciphertext\":\"AAAAAAAAAAAAAAAAAAAAAA==\"",
        ),
    ] {
        let missing = replace_once(&valid, member, "");
        assert_eq!(
            user_error(missing.as_bytes()),
            ExternalEnvelopeParseError::MissingField(field)
        );
    }

    for (input, expected) in [
        (
            r#"{"version":1,"\u0076ersion":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::DuplicateField(ExternalEnvelopeField::Version),
        ),
        (
            r#"{"version":1,"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::DuplicateField(ExternalEnvelopeField::Version),
        ),
        (
            r#"{"version":1,"algorithm":"AES/GCM/NoPadding","algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::DuplicateField(ExternalEnvelopeField::Algorithm),
        ),
        (
            r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::DuplicateField(ExternalEnvelopeField::Nonce),
        ),
        (
            r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA==","unknown":{}}"#,
            ExternalEnvelopeParseError::ForbiddenField,
        ),
        (
            r#"{"version":null,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::WrongFieldType(ExternalEnvelopeField::Version),
        ),
        (
            r#"{"version":1,"algorithm":null,"nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::WrongFieldType(ExternalEnvelopeField::Algorithm),
        ),
        (
            r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":7,"ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::WrongFieldType(ExternalEnvelopeField::Nonce),
        ),
        (
            r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":[]}"#,
            ExternalEnvelopeParseError::WrongFieldType(ExternalEnvelopeField::Ciphertext),
        ),
        (
            r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA==","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#,
            ExternalEnvelopeParseError::DuplicateField(ExternalEnvelopeField::Ciphertext),
        ),
    ] {
        assert_eq!(user_error(input.as_bytes()), expected, "{input}");
    }

    for version in ["0", "-1"] {
        let input = valid.replace("\"version\":1", &format!("\"version\":{version}"));
        assert_eq!(
            user_error(input.as_bytes()),
            ExternalEnvelopeParseError::InvalidVersion
        );
    }
    let null_nonce = valid.replace("\"nonce\":\"AAAAAAAAAAAAAAAA\"", "\"nonce\":null");
    assert_eq!(
        user_error(null_nonce.as_bytes()),
        ExternalEnvelopeParseError::WrongFieldType(ExternalEnvelopeField::Nonce)
    );
    let null_ciphertext = valid.replace(
        "\"ciphertext\":\"AAAAAAAAAAAAAAAAAAAAAA==\"",
        "\"ciphertext\":null",
    );
    assert_eq!(
        user_error(null_ciphertext.as_bytes()),
        ExternalEnvelopeParseError::WrongFieldType(ExternalEnvelopeField::Ciphertext)
    );
    let future = valid.replace("\"version\":1", "\"version\":2");
    assert_eq!(
        user_error(future.as_bytes()),
        ExternalEnvelopeParseError::UnsupportedVersion
    );
    let fractional = valid.replace("\"version\":1", "\"version\":1.5");
    assert_eq!(
        user_error(fractional.as_bytes()),
        ExternalEnvelopeParseError::NonIntegralVersion
    );
    let out_of_range = valid.replace("\"version\":1", "\"version\":18446744073709551616");
    assert_eq!(
        user_error(out_of_range.as_bytes()),
        ExternalEnvelopeParseError::VersionOutOfRange
    );
    let long_exact = format!("1.{}e0", "0".repeat(100_000));
    let input = valid.replace("\"version\":1", &format!("\"version\":{long_exact}"));
    assert_eq!(
        parse_encrypted_user_registry(input.as_bytes())
            .expect("long mathematically exact token should not require a copied number String")
            .version(),
        1
    );
    let algorithm = valid.replace(ALGORITHM, "AES/GCM/NoPadding-test-secret-sentinel");
    assert_eq!(
        user_error(algorithm.as_bytes()),
        ExternalEnvelopeParseError::InvalidAlgorithm
    );
}

#[test]
fn c01_c15_rejects_non_strict_json_and_invalid_text() {
    for (input, expected) in [
        ("", ExternalEnvelopeParseError::InvalidJson),
        (" \t\r\n", ExternalEnvelopeParseError::InvalidJson),
        ("[]", ExternalEnvelopeParseError::RootNotObject),
        ("true", ExternalEnvelopeParseError::RootNotObject),
        ("{\"version\":1,}", ExternalEnvelopeParseError::InvalidJson),
        ("{'version':1}", ExternalEnvelopeParseError::InvalidJson),
        ("{version:1}", ExternalEnvelopeParseError::InvalidJson),
        (
            "{/*comment*/\"version\":1}",
            ExternalEnvelopeParseError::InvalidJson,
        ),
        (
            "{\"version\":1,\"algorithm\":\"AES/GCM/NoPadding\",\"nonce\":\"AAAAAAAAAAAAAAAA\",\"ciphertext\":\"AAAAAAAAAAAAAAAAAAAAAA==\"}\u{00A0}",
            ExternalEnvelopeParseError::InvalidJson,
        ),
        (
            "{\"version\":1,\"algorithm\":\"AES/GCM/NoPadding\",\"nonce\":\"AAAAAAAAAAAAAAAA\",\"ciphertext\":\"AAAAAAAAAAAAAAAAAAAAAA==\"} trailing",
            ExternalEnvelopeParseError::InvalidJson,
        ),
        (
            "{\"version\":1,\"algorithm\":\"AES/GCM/NoPadding\",\"nonce\":\"AAAAAAAAAAAAAAAA\",\"ciphertext\":\"AAAAAAAAAAAAAAAAAAAAAA==\"} {}",
            ExternalEnvelopeParseError::InvalidJson,
        ),
    ] {
        assert_eq!(user_error(input.as_bytes()), expected, "{input}");
    }
    assert_eq!(
        user_error(&[0xEF, 0xBB, 0xBF, b'{']),
        ExternalEnvelopeParseError::InvalidBom
    );
    assert_eq!(
        user_error(b"{\"version\":1,\xFF}"),
        ExternalEnvelopeParseError::InvalidUtf8
    );
    assert_eq!(
        user_error(b"{\"version\":1,\"algorithm\":\"x"),
        ExternalEnvelopeParseError::InvalidJson
    );
    assert_eq!(user_error(b"\xC3"), ExternalEnvelopeParseError::InvalidUtf8);
    assert_eq!(
        user_error(br#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"\uD800"}"#),
        ExternalEnvelopeParseError::InvalidJson
    );
    assert_eq!(
        user_error(br#"{"version":1,"algorithm":"\uD800","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#),
        ExternalEnvelopeParseError::InvalidJson
    );
    for invalid_escape in [r#"\uDC00"#, r#"\uD800x"#, r#"\u12G4"#] {
        let input = format!(
            r#"{{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"{invalid_escape}"}}"#
        );
        assert_eq!(
            user_error(input.as_bytes()),
            ExternalEnvelopeParseError::InvalidJson
        );
    }
}

#[test]
fn b09_b10_b14_rejects_forbidden_fields_before_walking_their_values() {
    for input in [
        r#"{"forbidden":"x","version":1}"#,
        r#"{"version":1,"forbidden":{"nested":[1,{"x":2}]}}"#,
        r#"{"version":1,"forbidden":[1,2,3]}"#,
        r#"{"version":1,"forbidden":{"deep":{"deep":{"deep":{"deep":[]}}}}}"#,
        r#"{"version":1,"forbidden":{"malformed": }}"#,
        r#"{"version":1,"\u0066orbidden":null}"#,
        r#"{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA==","forbidden":0}"#,
    ] {
        assert_eq!(
            user_error(input.as_bytes()),
            ExternalEnvelopeParseError::ForbiddenField
        );
    }

    let long_name = "x".repeat(1_000_000);
    let input = format!(r#"{{"{long_name}":{{"ignored":[{{"deep":true}}]}}}}"#);
    assert_eq!(
        user_error(input.as_bytes()),
        ExternalEnvelopeParseError::ForbiddenField
    );

    let escaped_long_name = format!("\\u0078{}", "\\u0078".repeat(100_000));
    let input = format!(r#"{{"{escaped_long_name}":0}}"#);
    assert_eq!(
        user_error(input.as_bytes()),
        ExternalEnvelopeParseError::ForbiddenField
    );

    let escaped_known = r#"{"\u0076ersion":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#;
    assert!(parse_encrypted_user_registry(escaped_known.as_bytes()).is_ok());

    let escaped_unknown = r#"{"\u0075nknown":0}"#;
    assert_eq!(
        user_error(escaped_unknown.as_bytes()),
        ExternalEnvelopeParseError::ForbiddenField
    );
}

#[test]
fn d01_d10_e01_e10_enforces_canonical_base64_and_decoded_lengths() {
    let valid = valid_envelope_with_len(16);
    for (ciphertext, expected) in [
        ("", ExternalEnvelopeParseError::CiphertextTooShort),
        ("AA==", ExternalEnvelopeParseError::CiphertextTooShort),
        (
            "AAAAAAAAAAAAAAAAAAAAAA",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
        (
            "AAAAAAAAAAAAAAAAAAAAAA===",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
        (
            "AAAAAAAAAAAAAAAAAAAAAB==",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
        (
            "______________________==",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
        (
            "AAAA AAAA",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
        (
            "!!!!!!!!AAAAAAAAAAAAAAAA",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
        (
            "AAAAAAAAAAAAAAAAAAAAA===",
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext),
        ),
    ] {
        let input = valid.replace("AAAAAAAAAAAAAAAAAAAAAA==", ciphertext);
        assert_eq!(user_error(input.as_bytes()), expected, "{ciphertext}");
    }

    for nonce_len in [11, 13] {
        let input = valid.replace("AAAAAAAAAAAAAAAA", &b64(&vec![0; nonce_len]));
        assert_eq!(
            user_error(input.as_bytes()),
            ExternalEnvelopeParseError::InvalidNonceLength
        );
    }
    let invalid_nonce = valid.replace("AAAAAAAAAAAAAAAA", "!!!!!!!!!!!!!!!!");
    assert_eq!(
        user_error(invalid_nonce.as_bytes()),
        ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Nonce)
    );
    for ciphertext_len in [0, 1, 15] {
        let input = valid_envelope_with_len(ciphertext_len);
        assert_eq!(
            user_error(input.as_bytes()),
            ExternalEnvelopeParseError::CiphertextTooShort
        );
    }
}

#[test]
fn d09_e_scanner_decodes_escaped_base64_without_changing_its_value() {
    let ciphertext = b64(&[0; 16]);
    let escaped: String = ciphertext
        .chars()
        .map(|ch| format!(r#"\u{:04X}"#, ch as u32))
        .collect();
    let input = format!(
        r#"{{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"{escaped}"}}"#
    );
    let parsed = parse_encrypted_user_registry(input.as_bytes())
        .expect("escaped ASCII Base64 should decode to the same synthetic bytes");
    assert_eq!(parsed.ciphertext_and_tag(), &[0; 16]);

    let slash_bytes = [0xFF; 16];
    let slash_encoded = b64(&slash_bytes);
    let escaped_slashes = slash_encoded.replace('/', r#"\/"#);
    let escaped_padding = escaped_slashes.replace("==", r#"\u003D\u003D"#);
    let input = format!(
        r#"{{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"{escaped_padding}"}}"#
    );
    let parsed = parse_encrypted_user_registry(input.as_bytes())
        .expect("JSON escaped slash and padding should preserve canonical Base64 value");
    assert_eq!(parsed.ciphertext_and_tag(), &slash_bytes);

    // The Base64 padding quartet begins after the 4,092-byte decoder boundary.
    let boundary_ciphertext = vec![0x5A; 3_069];
    let boundary_encoded = b64(&boundary_ciphertext);
    assert_eq!(boundary_encoded.len(), 4_092);
    let boundary_escaped = format!("{}AA==", boundary_encoded);
    let boundary_json_escaped: String = boundary_escaped
        .chars()
        .map(|ch| format!(r#"\u{:04X}"#, ch as u32))
        .collect();
    let boundary_input = format!(
        r#"{{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"{boundary_json_escaped}"}}"#
    );
    let parsed = parse_encrypted_account_manifest(boundary_input.as_bytes())
        .expect("padding after a full decoder chunk should preserve the final quartet");
    let mut expected = boundary_ciphertext;
    expected.extend_from_slice(&[0; 1]);
    assert_eq!(parsed.ciphertext_and_tag(), expected);
}

#[test]
fn h_and_diagnostics_are_redacted_and_accessors_borrow() {
    let nonce = [0xD3; 12];
    let ciphertext = [0xA6; 16];
    let input = envelope("1", ALGORITHM, &b64(&nonce), &b64(&ciphertext));
    let parsed = parse_encrypted_user_registry(input.as_bytes()).unwrap();
    let debug = format!("{parsed:?}");
    assert!(!debug.contains(&b64(&nonce)));
    assert!(!debug.contains(&b64(&ciphertext)));
    let borrowed_ciphertext = parsed.ciphertext_and_tag();
    assert_eq!(borrowed_ciphertext, &ciphertext);
    assert_eq!(parsed.ciphertext_len(), ciphertext.len());

    let sentinel = "private-synthetic-field-sentinel";
    let input = format!(r#"{{"{sentinel}":0}}"#);
    let error = user_error(input.as_bytes());
    assert_eq!(error, ExternalEnvelopeParseError::ForbiddenField);
    assert!(!format!("{error:?}").contains(sentinel));
    assert!(!error.to_string().contains(sentinel));

    let invalid_algorithm =
        valid_envelope_with_len(16).replace(ALGORITHM, "algorithm-secret-sentinel");
    let error = user_error(invalid_algorithm.as_bytes());
    assert_eq!(error, ExternalEnvelopeParseError::InvalidAlgorithm);
    assert!(!format!("{error:?}").contains("algorithm-secret-sentinel"));
    assert!(!error.to_string().contains("algorithm-secret-sentinel"));

    let invalid_base64 = valid_envelope_with_len(16)
        .replace("AAAAAAAAAAAAAAAAAAAAAA==", "ciphertext-secret-sentinel");
    let error = user_error(invalid_base64.as_bytes());
    assert_eq!(
        error,
        ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext)
    );
    assert!(!format!("{error:?}").contains("ciphertext-secret-sentinel"));
    assert!(!error.to_string().contains("ciphertext-secret-sentinel"));

    let nonce_sentinel = "nonce-secret-sentinel";
    let input = valid_envelope_with_len(16).replace("AAAAAAAAAAAAAAAA", nonce_sentinel);
    let error = user_error(input.as_bytes());
    assert_eq!(
        error,
        ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Nonce)
    );
    assert!(!format!("{error:?}").contains(nonce_sentinel));
    assert!(!error.to_string().contains(nonce_sentinel));
}

#[test]
fn f01_f09_checks_full_file_limits_before_json_or_output_allocation() {
    const REGISTRY_MAX: usize = 16_777_216;
    const MANIFEST_MAX: usize = 67_108_864;

    assert_file_boundaries(
        REGISTRY_MAX,
        InputKind::EncryptedUserRegistry,
        12_582_843,
        12_582_846,
        parse_encrypted_user_registry,
    );
    assert_file_boundaries(
        MANIFEST_MAX,
        InputKind::EncryptedAccountManifest,
        50_331_579,
        50_331_582,
        parse_encrypted_account_manifest,
    );

    let oversized_for_registry = vec![b' '; REGISTRY_MAX + 1];
    assert_eq!(
        user_error(&oversized_for_registry),
        ExternalEnvelopeParseError::InputTooLarge(InputKind::EncryptedUserRegistry)
    );

    let fits_manifest_only = maximum_size_envelope(REGISTRY_MAX + 1, 1);
    assert_eq!(
        user_error(&fits_manifest_only),
        ExternalEnvelopeParseError::InputTooLarge(InputKind::EncryptedUserRegistry)
    );
    let parsed = parse_encrypted_account_manifest(&fits_manifest_only)
        .expect("the larger category must accept an otherwise-valid registry-sized envelope");
    assert!(parsed.ciphertext_len() > 12_000_000);
    drop(parsed);
    drop(fits_manifest_only);

    // Escaped input exercises the bounded staging path without constructing a full encoded String.
    const ESCAPED_CHARACTERS: usize = 1_000_000;
    let mut escaped_large = Vec::with_capacity(6 * ESCAPED_CHARACTERS + 128);
    escaped_large.extend_from_slice(
        b"{\"version\":1,\"algorithm\":\"AES/GCM/NoPadding\",\"nonce\":\"AAAAAAAAAAAAAAAA\",\"ciphertext\":\"",
    );
    for _ in 0..ESCAPED_CHARACTERS {
        escaped_large.extend_from_slice(b"\\u0041");
    }
    escaped_large.extend_from_slice(b"\"}");
    let parsed = parse_encrypted_account_manifest(&escaped_large)
        .expect("a large escaped canonical string should parse without a full encoded copy");
    assert_eq!(parsed.ciphertext_len(), ESCAPED_CHARACTERS / 4 * 3);
    drop(parsed);
    drop(escaped_large);

    const INVALID_LARGE_ENCODED: usize = 1_048_576;
    let mut invalid_large = maximum_size_envelope(INVALID_LARGE_ENCODED + 88, 0);
    let last_base64 = invalid_large.len() - 3;
    invalid_large[last_base64] = b'!';
    assert_eq!(
        user_error(&invalid_large),
        ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext)
    );
}

#[test]
fn d_and_g_reject_invalid_base64_near_the_end_without_panicking() {
    const ENCODED_LEN: usize = 16_384;
    let mut input = format!(
        r#"{{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"{}"}}"#,
        "A".repeat(ENCODED_LEN)
    )
    .into_bytes();
    let last = input.len() - 3;
    input[last] = b'!';
    for _ in 0..20 {
        let result = std::panic::catch_unwind(|| user_error(&input));
        assert_eq!(
            result.expect("malformed Base64 must not panic"),
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext)
        );
    }

    let invalid_length = format!(
        r#"{{"version":1,"algorithm":"AES/GCM/NoPadding","nonce":"AAAAAAAAAAAAAAAA","ciphertext":"{}"}}"#,
        "A".repeat(16_385)
    );
    assert_eq!(
        user_error(invalid_length.as_bytes()),
        ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext)
    );
}
