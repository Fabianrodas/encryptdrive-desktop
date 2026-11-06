use vault_crypto::aad::{
    file_content_aad, file_key_aad, manifest_aad, user_key_aad, users_aad, vault_key_aad,
};
use vault_format::{EntryId, UserId, VaultId};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn identifiers() -> (VaultId, UserId, EntryId) {
    (
        VaultId::from_canonical_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        UserId::from_canonical_str("123e4567-e89b-42d3-a456-426614174000").unwrap(),
        EntryId::from_canonical_str("f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap(),
    )
}

#[test]
fn aad_01_wrapped_registry_master_key_matches_exact_golden_bytes() {
    let (vault, _, _) = identifiers();
    let actual = vault_key_aad(&vault);
    let expected = b"EncryptDrive|vault-key|v1|550e8400-e29b-41d4-a716-446655440000";
    assert_eq!(actual.as_slice(), expected);
    assert_eq!(actual.len(), 62);
    assert_eq!(
        hex(&actual),
        "456e637279707444726976657c7661756c742d6b65797c76317c35353065383430302d653239622d343164342d613731362d343436363535343430303030"
    );
}

#[test]
fn aad_02_encrypted_user_registry_matches_exact_golden_bytes() {
    let (vault, _, _) = identifiers();
    let actual = users_aad(&vault);
    let expected = b"EncryptDrive|users|v1|550e8400-e29b-41d4-a716-446655440000";
    assert_eq!(actual.as_slice(), expected);
    assert_eq!(actual.len(), 58);
    assert_eq!(
        hex(&actual),
        "456e637279707444726976657c75736572737c76317c35353065383430302d653239622d343164342d613731362d343436363535343430303030"
    );
}

#[test]
fn aad_03_wrapped_user_master_key_matches_exact_golden_bytes() {
    let (vault, user, _) = identifiers();
    let actual = user_key_aad(&vault, &user);
    let expected = b"EncryptDrive|user-key|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000";
    assert_eq!(actual.as_slice(), expected);
    assert_eq!(actual.len(), 98);
    assert_eq!(
        hex(&actual),
        "456e637279707444726976657c757365722d6b65797c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d343236363134313734303030"
    );
}

#[test]
fn aad_04_encrypted_manifest_matches_exact_golden_bytes() {
    let (vault, user, _) = identifiers();
    let actual = manifest_aad(&vault, &user);
    let expected = b"EncryptDrive|manifest|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000";
    assert_eq!(actual.as_slice(), expected);
    assert_eq!(actual.len(), 98);
    assert_eq!(
        hex(&actual),
        "456e637279707444726976657c6d616e69666573747c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d343236363134313734303030"
    );
}

#[test]
fn aad_05_wrapped_file_key_uses_entry_id_and_matches_exact_golden_bytes() {
    let (vault, user, entry) = identifiers();
    let actual = file_key_aad(&vault, &user, &entry);
    let expected = b"EncryptDrive|file-key|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000|f47ac10b-58cc-4372-a567-0e02b2c3d479";
    assert_eq!(actual.as_slice(), expected);
    assert_eq!(actual.len(), 135);
    assert_eq!(
        hex(&actual),
        "456e637279707444726976657c66696c652d6b65797c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d3432363631343137343030307c66343761633130622d353863632d343337322d613536372d306530326232633364343739"
    );
}

#[test]
fn aad_06_encrypted_file_uses_entry_id_and_matches_exact_golden_bytes() {
    let (vault, user, entry) = identifiers();
    let actual = file_content_aad(&vault, &user, &entry);
    let expected = b"EncryptDrive|file|v1|550e8400-e29b-41d4-a716-446655440000|123e4567-e89b-42d3-a456-426614174000|f47ac10b-58cc-4372-a567-0e02b2c3d479";
    assert_eq!(actual.as_slice(), expected);
    assert_eq!(actual.len(), 131);
    assert_eq!(
        hex(&actual),
        "456e637279707444726976657c66696c657c76317c35353065383430302d653239622d343164342d613731362d3434363635353434303030307c31323365343536372d653839622d343264332d613435362d3432363631343137343030307c66343761633130622d353863632d343337322d613536372d306530326232633364343739"
    );
}
