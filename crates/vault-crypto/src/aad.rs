//! Pure VF1 associated-data formatting.
//!
//! These constructors return only the canonical UTF-8 bytes specified by VF1. They do not hash,
//! authenticate, encrypt, decrypt, derive keys, or access vault data.

use vault_format::{EntryId, UserId, VaultId};

/// Constructs AAD-01 for a wrapped registry master key.
pub fn vault_key_aad(vault_id: &VaultId) -> [u8; 62] {
    construct(
        b"EncryptDrive|vault-key|v1|",
        &[&vault_id.to_canonical_bytes()],
    )
}

/// Constructs AAD-02 for the encrypted user registry.
pub fn users_aad(vault_id: &VaultId) -> [u8; 58] {
    construct(b"EncryptDrive|users|v1|", &[&vault_id.to_canonical_bytes()])
}

/// Constructs AAD-03 for a wrapped user master key.
pub fn user_key_aad(vault_id: &VaultId, user_id: &UserId) -> [u8; 98] {
    construct(
        b"EncryptDrive|user-key|v1|",
        &[
            &vault_id.to_canonical_bytes(),
            &user_id.to_canonical_bytes(),
        ],
    )
}

/// Constructs AAD-04 for an encrypted manifest.
pub fn manifest_aad(vault_id: &VaultId, user_id: &UserId) -> [u8; 98] {
    construct(
        b"EncryptDrive|manifest|v1|",
        &[
            &vault_id.to_canonical_bytes(),
            &user_id.to_canonical_bytes(),
        ],
    )
}

/// Constructs AAD-05 for a wrapped file data-encryption key using the manifest `entryId`.
pub fn file_key_aad(vault_id: &VaultId, user_id: &UserId, entry_id: &EntryId) -> [u8; 135] {
    construct(
        b"EncryptDrive|file-key|v1|",
        &[
            &vault_id.to_canonical_bytes(),
            &user_id.to_canonical_bytes(),
            &entry_id.to_canonical_bytes(),
        ],
    )
}

/// Constructs AAD-06 for encrypted file content using the manifest `entryId`.
pub fn file_content_aad(vault_id: &VaultId, user_id: &UserId, entry_id: &EntryId) -> [u8; 131] {
    construct(
        b"EncryptDrive|file|v1|",
        &[
            &vault_id.to_canonical_bytes(),
            &user_id.to_canonical_bytes(),
            &entry_id.to_canonical_bytes(),
        ],
    )
}

fn construct<const N: usize>(prefix: &[u8], identifiers: &[&[u8; 36]]) -> [u8; N] {
    let mut output = [0_u8; N];
    let mut offset = 0;
    append(&mut output, &mut offset, prefix);
    for (index, identifier) in identifiers.iter().enumerate() {
        if index > 0 {
            append(&mut output, &mut offset, b"|");
        }
        append(&mut output, &mut offset, *identifier);
    }
    assert_eq!(offset, N, "AAD template length must match its return type");
    output
}

fn append<const N: usize>(output: &mut [u8; N], offset: &mut usize, bytes: &[u8]) {
    let end = offset
        .checked_add(bytes.len())
        .expect("fixed AAD size cannot overflow");
    output[*offset..end].copy_from_slice(bytes);
    *offset = end;
}
