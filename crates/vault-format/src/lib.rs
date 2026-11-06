#![forbid(unsafe_code)]
//! Zero-I/O structural validation for unauthenticated Vault Format 1 headers.
//!
//! [`parse_vault_header`] validates bounded caller-provided `vault.json` bytes and returns an
//! explicitly unauthenticated representation. It does not decrypt, authenticate, or perform
//! filesystem, network, operating-system, or secret access. [`LengthCheckedInput`] still records
//! only that its applicable size check passed; it does not itself represent parsed data.

use std::fmt;

mod external_envelope;
mod header;
mod identifiers;
mod java_case_fold;
mod logical_name;
mod strict_json;

pub use external_envelope::{
    ExternalEnvelopeField, ExternalEnvelopeParseError, UnauthenticatedExternalEnvelope,
    parse_encrypted_account_manifest, parse_encrypted_user_registry,
};
pub use header::{
    HeaderParseError, ParsedUnauthenticatedVaultHeader, StoredKdfAlgorithm, StoredKdfParameters,
    UnauthenticatedEncryptedEnvelope, VaultId, parse_vault_header,
};
pub use identifiers::{EntryId, IdentifierParseError, UserId};
pub use logical_name::{
    LogicalNameError, logical_name_eq_ignore_case, strip_java_whitespace, validate_logical_name,
};

/// The currently specified on-disk Vault Format version.
///
/// This constant identifies the currently supported header format version. The strict header
/// parser uses it to classify unsupported future versions after validating the full JSON input.
pub const VF1_FORMAT_VERSION: u8 = 1;

/// Maximum complete `vault.json` file size in bytes, including its wrapped-key envelope.
pub const VAULT_JSON_MAX_BYTES: usize = 256 * 1024;

/// Maximum complete encrypted user-registry file size in bytes, including its JSON envelope.
pub const ENCRYPTED_USER_REGISTRY_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Maximum complete encrypted account-manifest file size in bytes, including its JSON envelope.
pub const ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES: usize = 64 * 1024 * 1024;

/// A confirmed VF1 complete-file category used to select its byte-size limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// The complete plaintext `vault.json` header file.
    VaultHeader,
    /// The complete encrypted `users.enc` JSON-envelope file.
    EncryptedUserRegistry,
    /// The complete encrypted account-manifest JSON-envelope file.
    EncryptedAccountManifest,
}

impl InputKind {
    /// Returns the confirmed maximum complete-file size for this category, in bytes.
    pub const fn max_bytes(self) -> usize {
        match self {
            Self::VaultHeader => VAULT_JSON_MAX_BYTES,
            Self::EncryptedUserRegistry => ENCRYPTED_USER_REGISTRY_MAX_BYTES,
            Self::EncryptedAccountManifest => ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES,
        }
    }

    const fn file_label(self) -> &'static str {
        match self {
            Self::VaultHeader => "vault.json",
            Self::EncryptedUserRegistry => "encrypted user registry",
            Self::EncryptedAccountManifest => "encrypted account manifest",
        }
    }
}

/// A deterministic structural-boundary error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    /// The supplied byte length exceeds the confirmed limit for its complete-file category.
    InputTooLarge {
        /// The category whose limit was exceeded.
        kind: InputKind,
        /// The caller-supplied byte length.
        actual_bytes: usize,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { kind, actual_bytes } => write!(
                formatter,
                "{} is {actual_bytes} bytes; maximum is {maximum_bytes} bytes",
                kind.file_label(),
                maximum_bytes = kind.max_bytes()
            ),
        }
    }
}

impl std::error::Error for FormatError {}

/// Checks a caller-supplied byte length against a complete-file VF1 limit.
///
/// This numeric check allocates nothing and inspects no input content. Success means only that
/// the length is within the selected limit; it does not establish that the bytes are nonempty,
/// valid JSON, structurally valid metadata, or authenticated.
pub fn check_input_length(kind: InputKind, actual_bytes: usize) -> Result<(), FormatError> {
    let maximum_bytes = kind.max_bytes();
    if actual_bytes > maximum_bytes {
        return Err(FormatError::InputTooLarge { kind, actual_bytes });
    }

    Ok(())
}

/// Caller-supplied bytes that passed only the applicable full-file length check.
///
/// This wrapper borrows its input and makes no copy. Its name and API do not imply JSON parsing,
/// structural validation, or cryptographic authentication. Empty bytes pass the length check,
/// but are not thereby valid VF1 data.
#[derive(Clone, Copy)]
pub struct LengthCheckedInput<'a> {
    kind: InputKind,
    bytes: &'a [u8],
}

impl fmt::Debug for LengthCheckedInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LengthCheckedInput")
            .field("kind", &self.kind)
            .field("len", &self.bytes.len())
            .finish()
    }
}

impl<'a> LengthCheckedInput<'a> {
    /// Checks the byte slice's length and returns a borrowed wrapper when it is within the limit.
    pub fn new(kind: InputKind, bytes: &'a [u8]) -> Result<Self, FormatError> {
        check_input_length(kind, bytes.len())?;
        Ok(Self { kind, bytes })
    }

    /// Returns the complete-file category whose size limit was checked.
    pub const fn kind(&self) -> InputKind {
        self.kind
    }

    /// Returns the original caller-owned bytes without copying them.
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the checked byte length.
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether the checked byte slice is empty.
    ///
    /// An empty slice is within every size ceiling but is not a valid VF1 file.
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES, ENCRYPTED_USER_REGISTRY_MAX_BYTES, FormatError,
        InputKind, LengthCheckedInput, VAULT_JSON_MAX_BYTES, check_input_length,
    };
    use std::error::Error;

    #[test]
    fn header_limit_accepts_below_and_exact_and_rejects_above() {
        assert!(check_input_length(InputKind::VaultHeader, 262_143).is_ok());
        assert!(check_input_length(InputKind::VaultHeader, 262_144).is_ok());
        assert_eq!(
            check_input_length(InputKind::VaultHeader, 262_145),
            Err(FormatError::InputTooLarge {
                kind: InputKind::VaultHeader,
                actual_bytes: 262_145,
            })
        );
    }

    #[test]
    fn encrypted_registry_limit_accepts_below_and_exact_and_rejects_above() {
        assert!(check_input_length(InputKind::EncryptedUserRegistry, 16_777_215).is_ok());
        assert!(check_input_length(InputKind::EncryptedUserRegistry, 16_777_216).is_ok());
        assert_eq!(
            check_input_length(InputKind::EncryptedUserRegistry, 16_777_217),
            Err(FormatError::InputTooLarge {
                kind: InputKind::EncryptedUserRegistry,
                actual_bytes: 16_777_217,
            })
        );
    }

    #[test]
    fn encrypted_manifest_limit_accepts_below_and_exact_and_rejects_above() {
        assert!(check_input_length(InputKind::EncryptedAccountManifest, 67_108_863).is_ok());
        assert!(check_input_length(InputKind::EncryptedAccountManifest, 67_108_864).is_ok());
        assert_eq!(
            check_input_length(InputKind::EncryptedAccountManifest, 67_108_865),
            Err(FormatError::InputTooLarge {
                kind: InputKind::EncryptedAccountManifest,
                actual_bytes: 67_108_865,
            })
        );
    }

    #[test]
    fn zero_length_is_only_a_successful_size_check() {
        assert!(check_input_length(InputKind::VaultHeader, 0).is_ok());
        let checked = LengthCheckedInput::new(InputKind::VaultHeader, &[]).unwrap();
        assert!(checked.is_empty());
    }

    #[test]
    fn usize_max_is_rejected_without_overflow() {
        assert!(matches!(
            check_input_length(InputKind::EncryptedAccountManifest, usize::MAX),
            Err(FormatError::InputTooLarge {
                kind: InputKind::EncryptedAccountManifest,
                actual_bytes: usize::MAX,
            })
        ));
    }

    #[test]
    fn constructor_checks_oversized_borrowed_input_using_requested_kind() {
        let bytes = vec![0_u8; VAULT_JSON_MAX_BYTES + 1];

        assert_eq!(
            LengthCheckedInput::new(InputKind::VaultHeader, &bytes).unwrap_err(),
            FormatError::InputTooLarge {
                kind: InputKind::VaultHeader,
                actual_bytes: VAULT_JSON_MAX_BYTES + 1,
            }
        );

        let registry = LengthCheckedInput::new(InputKind::EncryptedUserRegistry, &bytes).unwrap();
        assert_eq!(registry.kind(), InputKind::EncryptedUserRegistry);
        assert_eq!(registry.as_bytes().as_ptr(), bytes.as_ptr());

        let manifest =
            LengthCheckedInput::new(InputKind::EncryptedAccountManifest, &bytes).unwrap();
        assert_eq!(manifest.kind(), InputKind::EncryptedAccountManifest);
        assert_eq!(manifest.as_bytes().as_ptr(), bytes.as_ptr());
    }

    #[test]
    fn constructor_accepts_exact_header_limit_and_rejects_one_byte_over() {
        let bytes = vec![0_u8; VAULT_JSON_MAX_BYTES + 1];

        let exact = LengthCheckedInput::new(InputKind::VaultHeader, &bytes[..VAULT_JSON_MAX_BYTES])
            .unwrap();
        assert_eq!(exact.kind(), InputKind::VaultHeader);
        assert_eq!(exact.len(), VAULT_JSON_MAX_BYTES);
        assert_eq!(
            exact.as_bytes().as_ptr(),
            bytes[..VAULT_JSON_MAX_BYTES].as_ptr()
        );

        assert_eq!(
            LengthCheckedInput::new(InputKind::VaultHeader, &bytes).unwrap_err(),
            FormatError::InputTooLarge {
                kind: InputKind::VaultHeader,
                actual_bytes: VAULT_JSON_MAX_BYTES + 1,
            }
        );
    }

    #[test]
    fn each_input_kind_uses_its_own_limit() {
        assert_eq!(VAULT_JSON_MAX_BYTES, 262_144);
        assert_eq!(ENCRYPTED_USER_REGISTRY_MAX_BYTES, 16_777_216);
        assert_eq!(ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES, 67_108_864);

        assert!(check_input_length(InputKind::VaultHeader, VAULT_JSON_MAX_BYTES).is_ok());
        assert!(
            check_input_length(InputKind::VaultHeader, ENCRYPTED_USER_REGISTRY_MAX_BYTES).is_err()
        );
        assert!(
            check_input_length(
                InputKind::EncryptedUserRegistry,
                ENCRYPTED_USER_REGISTRY_MAX_BYTES
            )
            .is_ok()
        );
        assert!(
            check_input_length(
                InputKind::EncryptedUserRegistry,
                ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES
            )
            .is_err()
        );
        assert!(
            check_input_length(
                InputKind::EncryptedAccountManifest,
                ENCRYPTED_ACCOUNT_MANIFEST_MAX_BYTES
            )
            .is_ok()
        );
    }

    #[test]
    fn input_too_large_display_reports_category_actual_and_max_deterministically() {
        let error = FormatError::InputTooLarge {
            kind: InputKind::VaultHeader,
            actual_bytes: VAULT_JSON_MAX_BYTES + 1,
        };
        let rendered = error.to_string();

        assert_eq!(
            rendered,
            "vault.json is 262145 bytes; maximum is 262144 bytes"
        );
        assert_eq!(rendered, error.to_string());
    }

    #[test]
    fn checked_input_borrows_the_original_bytes() {
        let source = [0x45, 0x44, 0x56];
        let checked = LengthCheckedInput::new(InputKind::VaultHeader, &source).unwrap();

        assert_eq!(checked.kind(), InputKind::VaultHeader);
        assert_eq!(checked.as_bytes(), source);
        assert_eq!(checked.as_bytes().as_ptr(), source.as_ptr());
        assert_eq!(checked.len(), source.len());
    }

    #[test]
    fn debug_for_checked_input_shows_metadata_without_borrowed_contents() {
        let sentinel = b"VF1-SECRET-PAYLOAD-MUST-NOT-APPEAR";
        let mut source = sentinel.to_vec();
        source.extend_from_slice(&[0x00, 0xFF]);
        let checked = LengthCheckedInput::new(InputKind::VaultHeader, &source).unwrap();
        let rendered = format!("{checked:?}");

        assert!(
            !rendered.contains(&format!("{source:?}")),
            "Debug output included the raw byte-array representation"
        );
        let hex_contents = source
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert!(
            !rendered.contains(&hex_contents),
            "Debug output included the input contents in hexadecimal"
        );
        assert!(
            !rendered.contains(std::str::from_utf8(sentinel).unwrap()),
            "Debug output included the synthetic sentinel"
        );
        assert!(rendered.contains("VaultHeader"));
        assert!(rendered.contains(&format!("len: {}", source.len())));
    }

    #[test]
    fn size_error_implements_standard_error() {
        fn assert_error<E: Error>() {}
        assert_error::<FormatError>();
    }
}
