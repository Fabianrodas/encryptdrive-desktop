use crate::identifiers::{parse_canonical_uuid_bytes, write_uuid};
use crate::strict_json::{
    ExactNumberError, JsonError, JsonKind, SchemaObjectError, StrictDocument, ValueSpan,
    parse_exact_u64, schema_fields,
};
use crate::{FormatError, InputKind, VAULT_JSON_MAX_BYTES};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;
use std::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const KDF_FIELDS: [&str; 5] = [
    "algorithm",
    "memoryKiB",
    "iterations",
    "parallelism",
    "salt",
];
const ENVELOPE_FIELDS: [&str; 4] = ["version", "algorithm", "nonce", "ciphertext"];
const MAX_MEMORY_KIB: u64 = 1_048_576;

/// The structural header parser's result. It is not authenticated or trusted metadata.
///
/// Successful parsing establishes only that the bounded bytes contain a strict JSON document
/// whose known VF1 fields satisfy their structural and semantic rules. It does not validate a
/// password, decrypt an envelope, authenticate the vault, or prove recoverability.
pub struct ParsedUnauthenticatedVaultHeader {
    vault_id: VaultId,
    created_at: OffsetDateTime,
    kdf: StoredKdfParameters,
    wrapped_registry_key: UnauthenticatedEncryptedEnvelope,
}

impl ParsedUnauthenticatedVaultHeader {
    /// Returns the supported on-disk format version.
    pub const fn format_version(&self) -> u8 {
        crate::VF1_FORMAT_VERSION
    }

    /// Returns the syntactically validated, canonical vault identifier.
    pub const fn vault_id(&self) -> &VaultId {
        &self.vault_id
    }

    /// Returns the parsed UTC creation time.
    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    /// Returns structurally validated stored KDF metadata; no derivation has occurred.
    pub const fn kdf(&self) -> &StoredKdfParameters {
        &self.kdf
    }

    /// Returns the unauthenticated wrapped-key envelope bytes.
    pub const fn wrapped_registry_key(&self) -> &UnauthenticatedEncryptedEnvelope {
        &self.wrapped_registry_key
    }
}

impl fmt::Debug for ParsedUnauthenticatedVaultHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ParsedUnauthenticatedVaultHeader")
            .field("format_version", &self.format_version())
            .field("vault_id", &"[REDACTED]")
            .field("created_at", &"[REDACTED]")
            .field("kdf", &self.kdf)
            .field("wrapped_registry_key", &self.wrapped_registry_key)
            .finish()
    }
}

/// A canonical lowercase VF1 vault UUID represented as its 16 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VaultId([u8; 16]);

impl VaultId {
    /// Parses exactly 36-character canonical lowercase UUID text.
    ///
    /// This validates text representation only. It does not constrain UUID version or variant
    /// bits and does not generate an identifier.
    pub fn from_canonical_str(text: &str) -> Result<Self, crate::IdentifierParseError> {
        parse_canonical_uuid_bytes(text).map(Self)
    }

    /// Returns the 16 decoded UUID bytes.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns canonical lowercase UUID text in a fixed-size byte array.
    pub fn to_canonical_bytes(self) -> [u8; 36] {
        crate::identifiers::format_uuid_bytes(&self.0)
    }
}

impl fmt::Display for VaultId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_uuid(&self.0, formatter)
    }
}

impl fmt::Debug for VaultId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VaultId")
            .field("redacted", &true)
            .finish()
    }
}

/// The only KDF algorithm currently accepted by the VF1 header contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredKdfAlgorithm {
    /// Argon2id is named in the stored metadata; this parser does not execute it.
    Argon2id,
}

/// Validated KDF parameters stored in a VF1 header. No KDF operation has occurred.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StoredKdfParameters {
    algorithm: StoredKdfAlgorithm,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    salt: [u8; 16],
}

impl StoredKdfParameters {
    /// Returns the validated stored KDF algorithm.
    pub const fn algorithm(&self) -> StoredKdfAlgorithm {
        self.algorithm
    }

    /// Returns the validated memory cost in KiB.
    pub const fn memory_kib(&self) -> u32 {
        self.memory_kib
    }

    /// Returns the validated iteration count.
    pub const fn iterations(&self) -> u32 {
        self.iterations
    }

    /// Returns the validated lane count.
    pub const fn parallelism(&self) -> u32 {
        self.parallelism
    }

    /// Returns the 16-byte stored salt for a later, separately authorized KDF operation.
    pub const fn salt(&self) -> &[u8; 16] {
        &self.salt
    }

    /// Returns the stored salt length without revealing its contents.
    pub const fn salt_len(&self) -> usize {
        self.salt.len()
    }
}

impl fmt::Debug for StoredKdfParameters {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredKdfParameters")
            .field("algorithm", &self.algorithm)
            .field("memory_kib", &self.memory_kib)
            .field("iterations", &self.iterations)
            .field("parallelism", &self.parallelism)
            .field("salt_bytes", &self.salt.len())
            .finish()
    }
}

/// A structurally valid but unauthenticated AES-GCM envelope.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnauthenticatedEncryptedEnvelope {
    version: u8,
    nonce: [u8; 12],
    ciphertext: [u8; 48],
}

impl UnauthenticatedEncryptedEnvelope {
    /// Returns the validated envelope version.
    pub const fn version(&self) -> u8 {
        self.version
    }

    /// Returns the 12-byte nonce. No GCM operation has occurred.
    pub const fn nonce(&self) -> &[u8; 12] {
        &self.nonce
    }

    /// Returns the 48-byte ciphertext plus tag. No tag verification has occurred.
    pub const fn ciphertext(&self) -> &[u8; 48] {
        &self.ciphertext
    }

    /// Returns the nonce length without revealing its contents.
    pub const fn nonce_len(&self) -> usize {
        self.nonce.len()
    }

    /// Returns the ciphertext length without revealing its contents.
    pub const fn ciphertext_len(&self) -> usize {
        self.ciphertext.len()
    }
}

impl fmt::Debug for UnauthenticatedEncryptedEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnauthenticatedEncryptedEnvelope")
            .field("version", &self.version)
            .field("algorithm", &"AES/GCM/NoPadding")
            .field("nonce_bytes", &self.nonce.len())
            .field("ciphertext_bytes", &self.ciphertext.len())
            .finish()
    }
}

/// A deterministic, sanitized structural header parsing error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderParseError {
    /// The complete input exceeds the confirmed `vault.json` limit.
    InputTooLarge { actual_bytes: usize },
    /// The byte input is not valid UTF-8.
    InvalidUtf8,
    /// The input begins with an unsupported UTF-8 byte-order mark.
    InvalidBom,
    /// The input is not exactly one valid strict JSON document.
    InvalidJson,
    /// The JSON root value is not an object.
    RootNotObject,
    /// An object repeats a property name after JSON string decoding.
    DuplicateProperty,
    /// A required schema field is absent.
    MissingField(&'static str),
    /// A schema field has the wrong JSON value type.
    WrongFieldType(&'static str),
    /// A known schema object contains a property outside its exact schema.
    ForbiddenField,
    /// A higher format version was structurally identified but is unsupported.
    UnsupportedVersion(u64),
    /// Format version zero is invalid.
    InvalidFormatVersion,
    /// A numeric value is not mathematically integral.
    NonIntegralNumber(&'static str),
    /// A numeric value is negative where prohibited or exceeds its supported range.
    NumericOutOfRange(&'static str),
    /// A vault ID is not in canonical lowercase 8-4-4-4-12 form.
    InvalidIdentifier,
    /// The timestamp is not a valid canonical UTC instant.
    InvalidTimestamp,
    /// Stored KDF metadata is outside the VF1 accepted values.
    InvalidKdfParameters(&'static str),
    /// A Base64 value is not canonical or does not decode to the exact expected length.
    InvalidBase64(&'static str),
    /// The nested encrypted envelope has an invalid supported value.
    InvalidEnvelope(&'static str),
    /// The JSON document exceeds the parser's approved container nesting depth.
    DepthLimitExceeded,
}

impl fmt::Display for HeaderParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { actual_bytes } => write!(
                formatter,
                "vault.json is {actual_bytes} bytes; maximum is {VAULT_JSON_MAX_BYTES} bytes"
            ),
            Self::InvalidUtf8 => formatter.write_str("vault.json is not valid UTF-8"),
            Self::InvalidBom => formatter.write_str("vault.json must not begin with a UTF-8 BOM"),
            Self::InvalidJson => formatter.write_str("vault.json is not one strict JSON document"),
            Self::RootNotObject => formatter.write_str("vault.json root must be an object"),
            Self::DuplicateProperty => {
                formatter.write_str("vault.json contains a duplicate object property")
            }
            Self::MissingField(field) => {
                write!(formatter, "vault.json is missing required field {field}")
            }
            Self::WrongFieldType(field) => write!(
                formatter,
                "vault.json field {field} has the wrong JSON type"
            ),
            Self::ForbiddenField => {
                formatter.write_str("a schema object contains an unsupported field")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "vault format version {version} is unsupported")
            }
            Self::InvalidFormatVersion => formatter.write_str("vault format version is invalid"),
            Self::NonIntegralNumber(field) => write!(
                formatter,
                "vault.json field {field} is not an exact integer"
            ),
            Self::NumericOutOfRange(field) => write!(
                formatter,
                "vault.json field {field} is outside its supported range"
            ),
            Self::InvalidIdentifier => formatter.write_str("vault.json vaultId is not canonical"),
            Self::InvalidTimestamp => {
                formatter.write_str("vault.json createdAt is not a canonical UTC timestamp")
            }
            Self::InvalidKdfParameters(field) => {
                write!(formatter, "vault.json KDF field {field} is invalid")
            }
            Self::InvalidBase64(field) => {
                write!(formatter, "vault.json Base64 field {field} is invalid")
            }
            Self::InvalidEnvelope(field) => {
                write!(formatter, "vault.json envelope field {field} is invalid")
            }
            Self::DepthLimitExceeded => {
                formatter.write_str("vault.json exceeds the maximum JSON nesting depth")
            }
        }
    }
}

impl std::error::Error for HeaderParseError {}

/// Parses a caller-owned, bounded `vault.json` byte slice into unauthenticated structural data.
///
/// The complete JSON document is strictly scanned before any schema or version decision. Unknown
/// top-level fields are validated and discarded; the known KDF and wrapped-key objects are exact.
/// This function performs no filesystem or network access and no cryptographic operation.
pub fn parse_vault_header(
    bytes: &[u8],
) -> Result<ParsedUnauthenticatedVaultHeader, HeaderParseError> {
    let checked = crate::LengthCheckedInput::new(InputKind::VaultHeader, bytes).map_err(
        |error| match error {
            FormatError::InputTooLarge { actual_bytes, .. } => {
                HeaderParseError::InputTooLarge { actual_bytes }
            }
        },
    )?;
    let bytes = checked.as_bytes();
    let document = StrictDocument::parse(bytes).map_err(map_json_error)?;
    if document.root_kind != JsonKind::Object {
        return Err(HeaderParseError::RootNotObject);
    }

    let version_span = required(document.fields.format_version, "formatVersion")?;
    let version = parse_u64(bytes, version_span, "formatVersion")?;
    if version == 0 {
        return Err(HeaderParseError::InvalidFormatVersion);
    }
    if version > u64::from(crate::VF1_FORMAT_VERSION) {
        return Err(HeaderParseError::UnsupportedVersion(version));
    }

    let vault_id = parse_vault_id(bytes, required(document.fields.vault_id, "vaultId")?)?;
    let created_at = parse_timestamp(bytes, required(document.fields.created_at, "createdAt")?)?;
    let kdf_span = required(document.fields.kdf, "kdf")?;
    let envelope_span = required(document.fields.wrapped_registry_key, "wrappedRegistryKey")?;
    let kdf = parse_kdf(bytes, kdf_span)?;
    let wrapped_registry_key = parse_wrapped_registry_key(bytes, envelope_span)?;

    Ok(ParsedUnauthenticatedVaultHeader {
        vault_id,
        created_at,
        kdf,
        wrapped_registry_key,
    })
}

fn map_json_error(error: JsonError) -> HeaderParseError {
    match error {
        JsonError::InvalidUtf8 => HeaderParseError::InvalidUtf8,
        JsonError::Bom => HeaderParseError::InvalidBom,
        JsonError::InvalidJson => HeaderParseError::InvalidJson,
        JsonError::DuplicateProperty => HeaderParseError::DuplicateProperty,
        JsonError::DepthLimitExceeded => HeaderParseError::DepthLimitExceeded,
    }
}

fn map_schema_error(error: SchemaObjectError) -> HeaderParseError {
    match error {
        SchemaObjectError::InvalidJson => HeaderParseError::InvalidJson,
        SchemaObjectError::ForbiddenField => HeaderParseError::ForbiddenField,
        SchemaObjectError::DuplicateProperty => HeaderParseError::DuplicateProperty,
    }
}

fn required(span: Option<ValueSpan>, field: &'static str) -> Result<ValueSpan, HeaderParseError> {
    span.ok_or(HeaderParseError::MissingField(field))
}

fn parse_kdf(bytes: &[u8], span: ValueSpan) -> Result<StoredKdfParameters, HeaderParseError> {
    if span.kind != JsonKind::Object {
        return Err(HeaderParseError::WrongFieldType("kdf"));
    }
    let fields = schema_fields(bytes, span, 2, KDF_FIELDS).map_err(map_schema_error)?;
    let algorithm = parse_string(bytes, required(fields[0], "algorithm")?, "algorithm")?;
    if algorithm != "Argon2id" {
        return Err(HeaderParseError::InvalidKdfParameters("algorithm"));
    }

    let memory_kib = parse_u64(bytes, required(fields[1], "memoryKiB")?, "memoryKiB")?;
    let iterations = parse_u64(bytes, required(fields[2], "iterations")?, "iterations")?;
    let parallelism = parse_u64(bytes, required(fields[3], "parallelism")?, "parallelism")?;

    if !(1..=8).contains(&parallelism) {
        return Err(HeaderParseError::InvalidKdfParameters("parallelism"));
    }
    if !(1..=10).contains(&iterations) {
        return Err(HeaderParseError::InvalidKdfParameters("iterations"));
    }
    let minimum_memory = parallelism
        .checked_mul(8)
        .ok_or(HeaderParseError::InvalidKdfParameters("memoryKiB"))?;
    if memory_kib < minimum_memory || memory_kib > MAX_MEMORY_KIB {
        return Err(HeaderParseError::InvalidKdfParameters("memoryKiB"));
    }

    let salt = decode_fixed::<16>(bytes, required(fields[4], "salt")?, "salt")?;

    Ok(StoredKdfParameters {
        algorithm: StoredKdfAlgorithm::Argon2id,
        memory_kib: u32::try_from(memory_kib)
            .map_err(|_| HeaderParseError::NumericOutOfRange("memoryKiB"))?,
        iterations: u32::try_from(iterations)
            .map_err(|_| HeaderParseError::NumericOutOfRange("iterations"))?,
        parallelism: u32::try_from(parallelism)
            .map_err(|_| HeaderParseError::NumericOutOfRange("parallelism"))?,
        salt,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct EnvelopeFieldSpans {
    pub(crate) version: ValueSpan,
    pub(crate) algorithm: ValueSpan,
    pub(crate) nonce: ValueSpan,
    pub(crate) ciphertext: ValueSpan,
}

/// Parses the shared four-member envelope schema for this and a future authorized parser.
pub(crate) fn parse_envelope_fields(
    bytes: &[u8],
    span: ValueSpan,
    object_depth: usize,
) -> Result<EnvelopeFieldSpans, HeaderParseError> {
    if span.kind != JsonKind::Object {
        return Err(HeaderParseError::WrongFieldType("wrappedRegistryKey"));
    }
    let fields =
        schema_fields(bytes, span, object_depth, ENVELOPE_FIELDS).map_err(map_schema_error)?;
    Ok(EnvelopeFieldSpans {
        version: required(fields[0], "version")?,
        algorithm: required(fields[1], "algorithm")?,
        nonce: required(fields[2], "nonce")?,
        ciphertext: required(fields[3], "ciphertext")?,
    })
}

fn parse_wrapped_registry_key(
    bytes: &[u8],
    span: ValueSpan,
) -> Result<UnauthenticatedEncryptedEnvelope, HeaderParseError> {
    let fields = parse_envelope_fields(bytes, span, 2)?;
    let version = parse_u64(bytes, fields.version, "version")?;
    if version != 1 {
        return Err(HeaderParseError::InvalidEnvelope("version"));
    }
    let algorithm = parse_string(bytes, fields.algorithm, "algorithm")?;
    if algorithm != "AES/GCM/NoPadding" {
        return Err(HeaderParseError::InvalidEnvelope("algorithm"));
    }
    Ok(UnauthenticatedEncryptedEnvelope {
        version: 1,
        nonce: decode_fixed::<12>(bytes, fields.nonce, "nonce")?,
        ciphertext: decode_fixed::<48>(bytes, fields.ciphertext, "ciphertext")?,
    })
}

fn parse_vault_id(bytes: &[u8], span: ValueSpan) -> Result<VaultId, HeaderParseError> {
    let text = parse_string(bytes, span, "vaultId")?;
    let decoded =
        parse_canonical_uuid_bytes(&text).map_err(|_| HeaderParseError::InvalidIdentifier)?;
    Ok(VaultId(decoded))
}

fn parse_timestamp(bytes: &[u8], span: ValueSpan) -> Result<OffsetDateTime, HeaderParseError> {
    let text = parse_string(bytes, span, "createdAt")?;
    if !has_canonical_utc_timestamp_shape(&text) {
        return Err(HeaderParseError::InvalidTimestamp);
    }
    OffsetDateTime::parse(&text, &Rfc3339).map_err(|_| HeaderParseError::InvalidTimestamp)
}

fn has_canonical_utc_timestamp_shape(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return false;
    }
    if ![0..4, 5..7, 8..10, 11..13, 14..16, 17..19]
        .iter()
        .all(|range| bytes[range.clone()].iter().all(u8::is_ascii_digit))
    {
        return false;
    }

    let Some(second_digits) = bytes.get(17..19) else {
        return false;
    };
    let seconds = (second_digits[0] - b'0') * 10 + (second_digits[1] - b'0');
    if seconds > 59 {
        return false;
    }

    let z_position = bytes.len() - 1;
    if bytes[z_position] != b'Z' {
        return false;
    }
    if z_position == 19 {
        return true;
    }
    z_position > 20
        && z_position - 20 <= 9
        && bytes[19] == b'.'
        && bytes[20..z_position].iter().all(u8::is_ascii_digit)
}

fn parse_string(
    bytes: &[u8],
    span: ValueSpan,
    field: &'static str,
) -> Result<String, HeaderParseError> {
    if span.kind != JsonKind::String {
        return Err(HeaderParseError::WrongFieldType(field));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(&bytes[span.start..span.end]);
    let value =
        String::deserialize(&mut deserializer).map_err(|_| HeaderParseError::InvalidJson)?;
    deserializer
        .end()
        .map_err(|_| HeaderParseError::InvalidJson)?;
    Ok(value)
}

fn parse_u64(bytes: &[u8], span: ValueSpan, field: &'static str) -> Result<u64, HeaderParseError> {
    if span.kind != JsonKind::Number {
        return Err(HeaderParseError::WrongFieldType(field));
    }

    parse_exact_u64(bytes, span).map_err(|kind| match kind {
        ExactNumberError::Invalid => HeaderParseError::InvalidJson,
        ExactNumberError::NonIntegral => HeaderParseError::NonIntegralNumber(field),
        ExactNumberError::OutOfRange => HeaderParseError::NumericOutOfRange(field),
    })
}

fn decode_fixed<const N: usize>(
    bytes: &[u8],
    span: ValueSpan,
    field: &'static str,
) -> Result<[u8; N], HeaderParseError> {
    if span.kind != JsonKind::String {
        return Err(HeaderParseError::WrongFieldType(field));
    }
    let expected_encoded_len = N.div_ceil(3) * 4;
    let mut encoded = [0_u8; 64];
    if expected_encoded_len > encoded.len() {
        return Err(HeaderParseError::InvalidBase64(field));
    }
    decode_json_ascii_string(bytes, span, &mut encoded[..expected_encoded_len])
        .map_err(|()| HeaderParseError::InvalidBase64(field))?;
    let encoded = std::str::from_utf8(&encoded[..expected_encoded_len])
        .map_err(|_| HeaderParseError::InvalidBase64(field))?;
    let mut decoded = [0_u8; N];
    match STANDARD.decode_slice(encoded, &mut decoded) {
        Ok(decoded_len) if decoded_len == N => Ok(decoded),
        _ => Err(HeaderParseError::InvalidBase64(field)),
    }
}

/// Decodes the JSON string spelling into a fixed ASCII buffer without allocating based on input.
/// The complete string has already passed the strict JSON scanner, but escapes are decoded here
/// because escaped `/` and ASCII Base64 characters have the same JSON string value.
fn decode_json_ascii_string(bytes: &[u8], span: ValueSpan, output: &mut [u8]) -> Result<(), ()> {
    if span.kind != JsonKind::String || span.end < span.start + 2 {
        return Err(());
    }
    let content = bytes.get(span.start + 1..span.end - 1).ok_or(())?;
    let mut input_index = 0;
    let mut output_index = 0;
    while input_index < content.len() {
        let byte = match content[input_index] {
            b'\\' => {
                input_index += 1;
                match content.get(input_index).copied().ok_or(())? {
                    b'/' => {
                        input_index += 1;
                        b'/'
                    }
                    b'u' => {
                        let digits = content.get(input_index + 1..input_index + 5).ok_or(())?;
                        let code_unit = decode_hex_u16(digits).ok_or(())?;
                        input_index += 5;
                        if (0xD800..=0xDFFF).contains(&code_unit) || code_unit > 0x7F {
                            return Err(());
                        }
                        code_unit as u8
                    }
                    _ => return Err(()),
                }
            }
            byte if byte.is_ascii() => {
                input_index += 1;
                byte
            }
            _ => return Err(()),
        };
        let target = output.get_mut(output_index).ok_or(())?;
        *target = byte;
        output_index += 1;
    }
    if output_index == output.len() {
        Ok(())
    } else {
        Err(())
    }
}

fn decode_hex_u16(digits: &[u8]) -> Option<u16> {
    if digits.len() != 4 {
        return None;
    }
    digits.iter().try_fold(0_u16, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        Some(value * 16 + u16::from(digit))
    })
}
