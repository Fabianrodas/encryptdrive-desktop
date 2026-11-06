use vault_format::{EntryId, UserId, VaultId};

const CANONICAL: &str = "550e8400-e29b-41d4-a716-446655440000";

#[test]
fn id_01_accepts_canonical_lowercase_uuid_and_round_trips() {
    let vault = VaultId::from_canonical_str(CANONICAL).expect("canonical vault ID");
    let user = UserId::from_canonical_str(CANONICAL).expect("canonical user ID");
    let entry = EntryId::from_canonical_str(CANONICAL).expect("canonical entry ID");

    assert_eq!(vault.to_string(), CANONICAL);
    assert_eq!(user.to_string(), CANONICAL);
    assert_eq!(entry.to_string(), CANONICAL);
    assert_eq!(vault.as_bytes(), user.as_bytes());
    assert_eq!(user.as_bytes(), entry.as_bytes());
}

#[test]
fn id_02_rejects_noncanonical_text_without_imposing_version_or_variant_rules() {
    let invalid = [
        "550E8400-E29B-41D4-A716-446655440000",
        "550e8400-E29b-41D4-a716-446655440000",
        "550e8400e29b41d4a716446655440000",
        "550e8400--e29b-41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-446655440000-",
        "550e8400_e29b-41d4-a716-446655440000",
        "550e8400-e29b41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-44665544000",
        "550e8400-e29b-41d4-a716-4466554400000",
        "550e8400-e29b-41d4-a716-44665544000g",
        "{550e8400-e29b-41d4-a716-446655440000}",
        " 550e8400-e29b-41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-446655440000\n",
        "",
    ];

    for text in invalid {
        assert!(
            VaultId::from_canonical_str(text).is_err(),
            "VaultId accepted {text:?}"
        );
        assert!(
            UserId::from_canonical_str(text).is_err(),
            "UserId accepted {text:?}"
        );
        assert!(
            EntryId::from_canonical_str(text).is_err(),
            "EntryId accepted {text:?}"
        );
    }

    // Nil, version 1, version 4, unusual version bits and non-RFC variant bits remain valid
    // canonical UUID values. Parsing does not generate IDs or impose a v4 policy.
    for text in [
        "00000000-0000-0000-0000-000000000000",
        "550e8400-e29b-11d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-446655440000",
        "550e8400-e29b-f1d4-1716-446655440000",
        "ffffffff-ffff-ffff-ffff-ffffffffffff",
    ] {
        let parsed = VaultId::from_canonical_str(text).expect("canonical UUID independent of bits");
        assert_eq!(parsed.to_string(), text);
    }
}

#[test]
fn identifier_debug_output_does_not_render_identifier_text() {
    let id = UserId::from_canonical_str(CANONICAL).unwrap();
    assert!(!format!("{id:?}").contains(CANONICAL));
}
