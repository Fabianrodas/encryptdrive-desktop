#![forbid(unsafe_code)]
//! Zero-I/O structural foundation for future Vault Format 1 parsing.
//!
//! This crate currently provides only the known VF1 version constant and bounded byte-length
//! checks for complete metadata files. A [`LengthCheckedInput`] has passed only its applicable
//! size check; it has not been parsed, structurally validated, or cryptographically
//! authenticated. This crate does not access files, networks, operating-system APIs, or secrets.

use std::fmt;

/// The currently specified on-disk Vault Format version.
///
/// This constant identifies VF1 as version 1. It does not inspect or validate input bytes, and
/// does not classify unsupported versions; that belongs to a future parser milestone.
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
#[derive(Debug, Clone, Copy)]
pub struct LengthCheckedInput<'a> {
    kind: InputKind,
    bytes: &'a [u8],
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
        InputKind, LengthCheckedInput, VAULT_JSON_MAX_BYTES, VF1_FORMAT_VERSION,
        check_input_length,
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
    fn size_error_is_deterministic_and_does_not_include_input_content() {
        let first = check_input_length(InputKind::VaultHeader, 262_145).unwrap_err();
        let second = check_input_length(InputKind::VaultHeader, 262_145).unwrap_err();

        assert_eq!(first, second);
        assert_eq!(first.to_string(), second.to_string());
        assert!(!first.to_string().contains("PRIVATE-SENTINEL"));
    }

    #[test]
    fn numeric_size_check_does_not_require_an_input_allocation() {
        // The API accepts the length directly; boundary values above require no large buffers.
        assert!(check_input_length(InputKind::EncryptedAccountManifest, usize::MAX).is_err());
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
    fn checked_input_terminology_does_not_claim_structural_or_crypto_validation() {
        let checked = LengthCheckedInput::new(InputKind::VaultHeader, b"not parsed").unwrap();
        assert_eq!(checked.as_bytes(), b"not parsed");
        assert_eq!(VF1_FORMAT_VERSION, 1);
    }

    #[test]
    fn size_error_implements_standard_error() {
        fn assert_error<E: Error>() {}
        assert_error::<FormatError>();
    }
}
