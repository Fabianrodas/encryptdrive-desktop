use crate::strict_json::{
    ExactNumberError, ExternalEnvelopeJsonError, ValueSpan, json_string_equals_ascii,
    parse_exact_u64, parse_external_envelope_fields, visit_json_string_ascii,
};
use crate::{FormatError, InputKind, LengthCheckedInput};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::fmt;

const ENVELOPE_ALGORITHM: &str = "AES/GCM/NoPadding";
const BASE64_STAGE_BYTES: usize = 4093;
const BASE64_SCRATCH_BYTES: usize = 3072;

/// A public field name in the fixed encrypted-envelope schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalEnvelopeField {
    /// The envelope format version.
    Version,
    /// The authenticated-encryption algorithm identifier.
    Algorithm,
    /// The 12-byte nonce.
    Nonce,
    /// The variable-length ciphertext followed by its authentication tag.
    Ciphertext,
}

impl ExternalEnvelopeField {
    const fn name(self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Algorithm => "algorithm",
            Self::Nonce => "nonce",
            Self::Ciphertext => "ciphertext",
        }
    }

    const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Version),
            1 => Some(Self::Algorithm),
            2 => Some(Self::Nonce),
            3 => Some(Self::Ciphertext),
            _ => None,
        }
    }
}

/// A sanitized structural parsing error for external encrypted envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalEnvelopeParseError {
    /// The complete input exceeds the established limit for its file category.
    InputTooLarge(InputKind),
    /// The bytes are not valid UTF-8.
    InvalidUtf8,
    /// The input begins with a UTF-8 byte-order mark.
    InvalidBom,
    /// The input is not one complete strict JSON document.
    InvalidJson,
    /// The JSON root is not an object.
    RootNotObject,
    /// A required field is absent.
    MissingField(ExternalEnvelopeField),
    /// A schema field occurs more than once after JSON escape decoding.
    DuplicateField(ExternalEnvelopeField),
    /// The object contains a property outside the fixed external-envelope schema.
    ForbiddenField,
    /// A schema field has a JSON type different from the required type.
    WrongFieldType(ExternalEnvelopeField),
    /// The version is zero or negative.
    InvalidVersion,
    /// The version is mathematically fractional.
    NonIntegralVersion,
    /// The version is outside the supported integer range.
    VersionOutOfRange,
    /// The version is a valid integer but is not supported by this implementation.
    UnsupportedVersion,
    /// The algorithm identifier is not the exact supported value.
    InvalidAlgorithm,
    /// A Base64 string is invalid or noncanonical.
    InvalidBase64(ExternalEnvelopeField),
    /// The decoded nonce is not exactly 12 bytes.
    InvalidNonceLength,
    /// The decoded ciphertext is shorter than the 16-byte authentication tag.
    CiphertextTooShort,
    /// A required output allocation could not be reserved.
    AllocationFailed,
}

impl fmt::Display for ExternalEnvelopeParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge(InputKind::EncryptedUserRegistry) => {
                formatter.write_str("encrypted user registry exceeds its complete-file size limit")
            }
            Self::InputTooLarge(InputKind::EncryptedAccountManifest) => formatter
                .write_str("encrypted account manifest exceeds its complete-file size limit"),
            Self::InputTooLarge(InputKind::VaultHeader) => {
                formatter.write_str("external envelope exceeds its complete-file size limit")
            }
            Self::InvalidUtf8 => formatter.write_str("external envelope is not valid UTF-8"),
            Self::InvalidBom => {
                formatter.write_str("external envelope must not begin with a UTF-8 BOM")
            }
            Self::InvalidJson => {
                formatter.write_str("external envelope is not one strict JSON document")
            }
            Self::RootNotObject => formatter.write_str("external envelope root must be an object"),
            Self::MissingField(field) => {
                write!(
                    formatter,
                    "external envelope is missing required field {}",
                    field.name()
                )
            }
            Self::DuplicateField(field) => {
                write!(
                    formatter,
                    "external envelope repeats field {}",
                    field.name()
                )
            }
            Self::ForbiddenField => {
                formatter.write_str("external envelope contains an unsupported field")
            }
            Self::WrongFieldType(field) => {
                write!(
                    formatter,
                    "external envelope field {} has the wrong JSON type",
                    field.name()
                )
            }
            Self::InvalidVersion => formatter.write_str("external envelope version is invalid"),
            Self::NonIntegralVersion => {
                formatter.write_str("external envelope version is not an exact integer")
            }
            Self::VersionOutOfRange => {
                formatter.write_str("external envelope version is outside its supported range")
            }
            Self::UnsupportedVersion => {
                formatter.write_str("external envelope version is unsupported")
            }
            Self::InvalidAlgorithm => {
                formatter.write_str("external envelope algorithm is unsupported")
            }
            Self::InvalidBase64(field) => {
                write!(
                    formatter,
                    "external envelope field {} is not canonical Base64",
                    field.name()
                )
            }
            Self::InvalidNonceLength => {
                formatter.write_str("external envelope nonce must decode to 12 bytes")
            }
            Self::CiphertextTooShort => formatter
                .write_str("external envelope ciphertext is shorter than its authentication tag"),
            Self::AllocationFailed => {
                formatter.write_str("external envelope output allocation failed")
            }
        }
    }
}

impl std::error::Error for ExternalEnvelopeParseError {}

/// Structurally valid external-envelope bytes whose GCM authentication has not been checked.
pub struct UnauthenticatedExternalEnvelope {
    version: u32,
    nonce: [u8; 12],
    ciphertext_and_tag: Vec<u8>,
}

impl UnauthenticatedExternalEnvelope {
    /// Returns the structurally supported envelope version.
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Returns the nonce without copying it.
    pub const fn nonce(&self) -> &[u8; 12] {
        &self.nonce
    }

    /// Returns ciphertext plus authentication tag without copying it.
    pub fn ciphertext_and_tag(&self) -> &[u8] {
        &self.ciphertext_and_tag
    }

    /// Returns the decoded ciphertext-plus-tag length.
    pub const fn ciphertext_len(&self) -> usize {
        self.ciphertext_and_tag.len()
    }
}

impl fmt::Debug for UnauthenticatedExternalEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnauthenticatedExternalEnvelope")
            .field("version", &self.version)
            .field("nonce_len", &self.nonce.len())
            .field("ciphertext_and_tag_len", &self.ciphertext_and_tag.len())
            .field("authenticated", &false)
            .finish()
    }
}

/// Parses a caller-provided `users.enc`-category envelope without filesystem or crypto access.
pub fn parse_encrypted_user_registry(
    input: &[u8],
) -> Result<UnauthenticatedExternalEnvelope, ExternalEnvelopeParseError> {
    parse_for_kind(InputKind::EncryptedUserRegistry, input)
}

/// Parses a caller-provided manifest-category envelope without filesystem or crypto access.
pub fn parse_encrypted_account_manifest(
    input: &[u8],
) -> Result<UnauthenticatedExternalEnvelope, ExternalEnvelopeParseError> {
    parse_for_kind(InputKind::EncryptedAccountManifest, input)
}

fn parse_for_kind(
    kind: InputKind,
    input: &[u8],
) -> Result<UnauthenticatedExternalEnvelope, ExternalEnvelopeParseError> {
    let checked =
        LengthCheckedInput::new(kind, input).map_err(|FormatError::InputTooLarge { .. }| {
            ExternalEnvelopeParseError::InputTooLarge(kind)
        })?;
    parse_checked(checked)
}

fn parse_checked(
    checked: LengthCheckedInput<'_>,
) -> Result<UnauthenticatedExternalEnvelope, ExternalEnvelopeParseError> {
    let input = checked.as_bytes();
    let fields = parse_external_envelope_fields(input).map_err(map_json_error)?;

    if input[fields.version.start] == b'-' {
        return Err(ExternalEnvelopeParseError::InvalidVersion);
    }
    let version = parse_exact_u64(input, fields.version).map_err(map_number_error)?;
    match version {
        0 => return Err(ExternalEnvelopeParseError::InvalidVersion),
        1 => {}
        _ => return Err(ExternalEnvelopeParseError::UnsupportedVersion),
    }
    if !json_string_equals_ascii(input, fields.algorithm, ENVELOPE_ALGORITHM) {
        return Err(ExternalEnvelopeParseError::InvalidAlgorithm);
    }

    let nonce_len = validate_base64(input, fields.nonce)
        .map_err(|()| ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Nonce))?;
    if nonce_len != 12 {
        return Err(ExternalEnvelopeParseError::InvalidNonceLength);
    }
    let mut nonce = [0_u8; 12];
    let decoded_nonce = decode_base64(input, fields.nonce, &mut nonce)
        .map_err(|()| ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Nonce))?;
    if decoded_nonce != nonce.len() {
        return Err(ExternalEnvelopeParseError::InvalidNonceLength);
    }

    let ciphertext_len = validate_base64(input, fields.ciphertext).map_err(|()| {
        ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext)
    })?;
    if ciphertext_len < 16 {
        return Err(ExternalEnvelopeParseError::CiphertextTooShort);
    }
    let mut ciphertext_and_tag = Vec::new();
    ciphertext_and_tag
        .try_reserve_exact(ciphertext_len)
        .map_err(|_| ExternalEnvelopeParseError::AllocationFailed)?;
    ciphertext_and_tag.resize(ciphertext_len, 0);
    let decoded_ciphertext = decode_base64(input, fields.ciphertext, &mut ciphertext_and_tag)
        .map_err(|()| {
            ExternalEnvelopeParseError::InvalidBase64(ExternalEnvelopeField::Ciphertext)
        })?;
    if decoded_ciphertext != ciphertext_len {
        return Err(ExternalEnvelopeParseError::InvalidBase64(
            ExternalEnvelopeField::Ciphertext,
        ));
    }

    Ok(UnauthenticatedExternalEnvelope {
        version: version as u32,
        nonce,
        ciphertext_and_tag,
    })
}

fn map_number_error(error: ExactNumberError) -> ExternalEnvelopeParseError {
    match error {
        ExactNumberError::Invalid => ExternalEnvelopeParseError::InvalidJson,
        ExactNumberError::NonIntegral => ExternalEnvelopeParseError::NonIntegralVersion,
        ExactNumberError::OutOfRange => ExternalEnvelopeParseError::VersionOutOfRange,
    }
}

fn map_json_error(error: ExternalEnvelopeJsonError) -> ExternalEnvelopeParseError {
    match error {
        ExternalEnvelopeJsonError::InvalidUtf8 => ExternalEnvelopeParseError::InvalidUtf8,
        ExternalEnvelopeJsonError::Bom => ExternalEnvelopeParseError::InvalidBom,
        ExternalEnvelopeJsonError::InvalidJson => ExternalEnvelopeParseError::InvalidJson,
        ExternalEnvelopeJsonError::RootNotObject => ExternalEnvelopeParseError::RootNotObject,
        ExternalEnvelopeJsonError::ForbiddenField => ExternalEnvelopeParseError::ForbiddenField,
        ExternalEnvelopeJsonError::DuplicateField(index) => {
            ExternalEnvelopeField::from_index(index)
                .map(ExternalEnvelopeParseError::DuplicateField)
                .unwrap_or(ExternalEnvelopeParseError::InvalidJson)
        }
        ExternalEnvelopeJsonError::MissingField(index) => ExternalEnvelopeField::from_index(index)
            .map(ExternalEnvelopeParseError::MissingField)
            .unwrap_or(ExternalEnvelopeParseError::InvalidJson),
        ExternalEnvelopeJsonError::WrongFieldType(index) => {
            ExternalEnvelopeField::from_index(index)
                .map(ExternalEnvelopeParseError::WrongFieldType)
                .unwrap_or(ExternalEnvelopeParseError::InvalidJson)
        }
    }
}

fn validate_base64(input: &[u8], span: ValueSpan) -> Result<usize, ()> {
    if let Some(encoded) = unescaped_string(input, span) {
        return validate_encoded_base64(encoded);
    }

    visit_escaped_base64_chunks(input, span, |chunk| {
        let mut scratch = [0_u8; BASE64_SCRATCH_BYTES];
        STANDARD.decode_slice(chunk, &mut scratch).map_err(|_| ())
    })
}

fn validate_encoded_base64(encoded: &[u8]) -> Result<usize, ()> {
    if !encoded.len().is_multiple_of(4) {
        return Err(());
    }
    let mut decoded_len = 0_usize;
    let mut saw_padding = false;
    for (index, chunk) in encoded.chunks(4092).enumerate() {
        if saw_padding {
            return Err(());
        }
        let has_padding = chunk.contains(&b'=');
        if has_padding && index + 1 != encoded.len().div_ceil(4092) {
            return Err(());
        }
        let mut scratch = [0_u8; BASE64_SCRATCH_BYTES];
        let chunk_len = STANDARD.decode_slice(chunk, &mut scratch).map_err(|_| ())?;
        decoded_len = decoded_len.checked_add(chunk_len).ok_or(())?;
        saw_padding = has_padding;
    }
    Ok(decoded_len)
}

fn decode_base64(input: &[u8], span: ValueSpan, output: &mut [u8]) -> Result<usize, ()> {
    if let Some(encoded) = unescaped_string(input, span) {
        return STANDARD.decode_slice(encoded, output).map_err(|_| ());
    }

    let mut offset = 0_usize;
    let decoded_len = visit_escaped_base64_chunks(input, span, |chunk| {
        let target = output.get_mut(offset..).ok_or(())?;
        let written = STANDARD.decode_slice(chunk, target).map_err(|_| ())?;
        offset = offset.checked_add(written).ok_or(())?;
        Ok(written)
    })?;
    if offset != decoded_len {
        return Err(());
    }
    Ok(decoded_len)
}

fn visit_escaped_base64_chunks(
    input: &[u8],
    span: ValueSpan,
    mut decode_chunk: impl FnMut(&[u8]) -> Result<usize, ()>,
) -> Result<usize, ()> {
    let mut staging = [0_u8; BASE64_STAGE_BYTES];
    let mut buffered = 0_usize;
    let mut decoded_len = 0_usize;
    let mut saw_padding = false;
    let mut invalid = false;
    let completed = visit_json_string_ascii(input, span, |byte| {
        let Some(byte) = byte else {
            invalid = true;
            return false;
        };
        if saw_padding {
            invalid = true;
            return false;
        }
        staging[buffered] = byte;
        buffered += 1;
        if buffered == staging.len() {
            let complete_len = buffered - buffered % 4;
            let has_padding = staging[..complete_len].contains(&b'=');
            if has_padding && complete_len != buffered {
                invalid = true;
                return false;
            }
            let chunk_len = match decode_chunk(&staging[..complete_len]) {
                Ok(chunk_len) => chunk_len,
                Err(()) => {
                    invalid = true;
                    return false;
                }
            };
            decoded_len = match decoded_len.checked_add(chunk_len) {
                Some(decoded_len) => decoded_len,
                None => {
                    invalid = true;
                    return false;
                }
            };
            saw_padding = has_padding;
            let carry_len = buffered - complete_len;
            staging.copy_within(complete_len..buffered, 0);
            buffered = carry_len;
            if saw_padding && buffered != 0 {
                invalid = true;
                return false;
            }
        }
        true
    });
    if !completed || invalid || !buffered.is_multiple_of(4) || (saw_padding && buffered != 0) {
        return Err(());
    }
    if buffered != 0 {
        decoded_len = decoded_len
            .checked_add(decode_chunk(&staging[..buffered])?)
            .ok_or(())?;
    }
    Ok(decoded_len)
}

fn unescaped_string(input: &[u8], span: ValueSpan) -> Option<&[u8]> {
    if span.end < span.start + 2 || span.kind != crate::strict_json::JsonKind::String {
        return None;
    }
    let content = input.get(span.start + 1..span.end - 1)?;
    (!content.contains(&b'\\')).then_some(content)
}

#[cfg(test)]
mod tests {
    use super::visit_escaped_base64_chunks;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    #[test]
    fn escaped_base64_quartets_survive_the_staging_carry_boundary() {
        let encoded = "A".repeat(8_192) + "AA==";
        let escaped: String = encoded
            .chars()
            .map(|character| format!(r#"\u{:04X}"#, character as u32))
            .collect();
        let input = format!(r#""{escaped}""#);
        let span = crate::strict_json::ValueSpan {
            start: 0,
            end: input.len(),
            kind: crate::strict_json::JsonKind::String,
        };
        let mut output = Vec::new();
        let decoded_len = visit_escaped_base64_chunks(input.as_bytes(), span, |chunk| {
            let mut scratch = [0_u8; 3072];
            let len = STANDARD.decode_slice(chunk, &mut scratch).map_err(|_| ())?;
            output.extend_from_slice(&scratch[..len]);
            Ok(len)
        })
        .expect("canonical escaped chunks should decode");
        assert_eq!(output.len(), decoded_len);
        assert_eq!(decoded_len, 6_145);
    }
}
