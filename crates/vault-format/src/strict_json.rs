use serde::Deserialize;
use std::collections::HashSet;

pub(crate) const MAX_JSON_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonError {
    InvalidUtf8,
    Bom,
    InvalidJson,
    DuplicateProperty,
    DepthLimitExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: JsonKind,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct RootFields {
    pub(crate) format_version: Option<ValueSpan>,
    pub(crate) vault_id: Option<ValueSpan>,
    pub(crate) created_at: Option<ValueSpan>,
    pub(crate) kdf: Option<ValueSpan>,
    pub(crate) wrapped_registry_key: Option<ValueSpan>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictDocument {
    pub(crate) root_kind: JsonKind,
    pub(crate) fields: RootFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalEnvelopeJsonError {
    InvalidUtf8,
    Bom,
    InvalidJson,
    RootNotObject,
    ForbiddenField,
    DuplicateField(u8),
    MissingField(u8),
    WrongFieldType(u8),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExternalEnvelopeSpans {
    pub(crate) version: ValueSpan,
    pub(crate) algorithm: ValueSpan,
    pub(crate) nonce: ValueSpan,
    pub(crate) ciphertext: ValueSpan,
}

#[derive(Clone, Copy)]
enum RootKey {
    FormatVersion,
    VaultId,
    CreatedAt,
    Kdf,
    WrappedRegistryKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaObjectError {
    InvalidJson,
    ForbiddenField,
    DuplicateProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExactNumberError {
    Invalid,
    NonIntegral,
    OutOfRange,
}

/// Converts a validated JSON number span to an exact unsigned integer without a temporary String.
pub(crate) fn parse_exact_u64(input: &[u8], span: ValueSpan) -> Result<u64, ExactNumberError> {
    if span.kind != JsonKind::Number {
        return Err(ExactNumberError::Invalid);
    }
    let token = input
        .get(span.start..span.end)
        .ok_or(ExactNumberError::Invalid)?;
    let number = std::str::from_utf8(token).map_err(|_| ExactNumberError::Invalid)?;
    exact_u64(number)
}

fn exact_u64(number: &str) -> Result<u64, ExactNumberError> {
    let (negative, unsigned) = match number.strip_prefix('-') {
        Some(unsigned) => (true, unsigned),
        None => (false, number),
    };
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(index) => (&unsigned[..index], parse_exponent(&unsigned[index + 1..])?),
        None => (unsigned, 0),
    };
    let (whole, fraction) = match mantissa.find('.') {
        Some(index) => (&mantissa[..index], &mantissa[index + 1..]),
        None => (mantissa, ""),
    };
    let total_digits = whole
        .len()
        .checked_add(fraction.len())
        .ok_or(ExactNumberError::OutOfRange)?;
    let first_nonzero = whole.bytes().position(|byte| byte != b'0').or_else(|| {
        fraction
            .bytes()
            .position(|byte| byte != b'0')
            .map(|index| whole.len() + index)
    });
    let Some(first_nonzero) = first_nonzero else {
        return Ok(0);
    };
    let fraction_len = i64::try_from(fraction.len()).map_err(|_| ExactNumberError::OutOfRange)?;
    let scale = fraction_len
        .checked_sub(exponent)
        .ok_or(ExactNumberError::OutOfRange)?;

    let magnitude = if scale > 0 {
        let scale = usize::try_from(scale).map_err(|_| ExactNumberError::NonIntegral)?;
        if scale > total_digits {
            return Err(ExactNumberError::NonIntegral);
        }
        let integer_end = total_digits - scale;
        if (integer_end..total_digits)
            .any(|index| concatenated_digit(whole, fraction, index) != b'0')
        {
            return Err(ExactNumberError::NonIntegral);
        }
        parse_concatenated_digits(whole, fraction, first_nonzero, integer_end)?
    } else {
        let appended_zeroes =
            usize::try_from(scale.checked_neg().ok_or(ExactNumberError::OutOfRange)?)
                .map_err(|_| ExactNumberError::OutOfRange)?;
        let significant_len = total_digits - first_nonzero;
        if significant_len
            .checked_add(appended_zeroes)
            .ok_or(ExactNumberError::OutOfRange)?
            > 20
        {
            return Err(ExactNumberError::OutOfRange);
        }
        let mut value = parse_concatenated_digits(whole, fraction, first_nonzero, total_digits)?;
        for _ in 0..appended_zeroes {
            value = value.checked_mul(10).ok_or(ExactNumberError::OutOfRange)?;
        }
        value
    };

    if negative && magnitude != 0 {
        return Err(ExactNumberError::OutOfRange);
    }
    Ok(magnitude)
}

fn concatenated_digit(whole: &str, fraction: &str, index: usize) -> u8 {
    if index < whole.len() {
        whole.as_bytes()[index]
    } else {
        fraction.as_bytes()[index - whole.len()]
    }
}

fn parse_concatenated_digits(
    whole: &str,
    fraction: &str,
    start: usize,
    end: usize,
) -> Result<u64, ExactNumberError> {
    (start..end).try_fold(0_u64, |value, index| {
        value
            .checked_mul(10)
            .and_then(|value| {
                value.checked_add(u64::from(concatenated_digit(whole, fraction, index) - b'0'))
            })
            .ok_or(ExactNumberError::OutOfRange)
    })
}

fn parse_exponent(exponent: &str) -> Result<i64, ExactNumberError> {
    let (negative, digits) = if let Some(digits) = exponent.strip_prefix('-') {
        (true, digits)
    } else if let Some(digits) = exponent.strip_prefix('+') {
        (false, digits)
    } else {
        (false, exponent)
    };
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(0);
    }

    // Values beyond this cap are already outside the u64 precision/range of any nonzero field.
    let mut value = 0_i64;
    for byte in digits.bytes() {
        if !byte.is_ascii_digit() {
            return Err(ExactNumberError::OutOfRange);
        }
        value = value
            .saturating_mul(10)
            .saturating_add(i64::from(byte - b'0'))
            .min(1_000_000);
    }
    Ok(if negative { -value } else { value })
}

impl StrictDocument {
    /// Validates one complete JSON document and records only known root-value spans.
    /// Unknown values are recursively validated and discarded.
    pub(crate) fn parse(input: &[u8]) -> Result<Self, JsonError> {
        std::str::from_utf8(input).map_err(|_| JsonError::InvalidUtf8)?;
        if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return Err(JsonError::Bom);
        }

        let mut parser = Parser {
            input,
            position: 0,
            fields: RootFields::default(),
        };
        parser.whitespace();
        let root = parser.value(0, true)?;
        parser.whitespace();
        if parser.position != input.len() {
            return Err(JsonError::InvalidJson);
        }

        Ok(Self {
            root_kind: root.kind,
            fields: parser.fields,
        })
    }
}

/// Parses only the fixed external-envelope root schema.
///
/// Unknown fields and container-valued fields are rejected before their values are traversed.
/// A successful result means the entire root object and the document terminator were validated.
pub(crate) fn parse_external_envelope_fields(
    input: &[u8],
) -> Result<ExternalEnvelopeSpans, ExternalEnvelopeJsonError> {
    std::str::from_utf8(input).map_err(|_| ExternalEnvelopeJsonError::InvalidUtf8)?;
    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ExternalEnvelopeJsonError::Bom);
    }

    let mut parser = Parser {
        input,
        position: 0,
        fields: RootFields::default(),
    };
    parser.whitespace();
    match input.get(parser.position) {
        Some(b'{') => parser.position += 1,
        Some(b'[' | b'"' | b't' | b'f' | b'n' | b'-' | b'0'..=b'9') => {
            return Err(ExternalEnvelopeJsonError::RootNotObject);
        }
        _ => return Err(ExternalEnvelopeJsonError::InvalidJson),
    }

    let mut fields = [None; 4];
    parser.whitespace();
    let mut closed = false;
    if parser.input.get(parser.position) == Some(&b'}') {
        parser.position += 1;
        closed = true;
    }

    while !closed {
        let key = parser
            .string()
            .map_err(|_| ExternalEnvelopeJsonError::InvalidJson)?;
        let field_index = parser.external_envelope_field_index(key);
        parser.whitespace();
        if parser.input.get(parser.position) != Some(&b':') {
            return Err(ExternalEnvelopeJsonError::InvalidJson);
        }
        parser.position += 1;
        let field_index = field_index.ok_or(ExternalEnvelopeJsonError::ForbiddenField)?;
        if fields[field_index].is_some() {
            return Err(ExternalEnvelopeJsonError::DuplicateField(field_index as u8));
        }

        parser.whitespace();
        let value = parser
            .scalar_or_container_kind()
            .map_err(|_| ExternalEnvelopeJsonError::InvalidJson)?;
        let expected_kind = if field_index == 0 {
            JsonKind::Number
        } else {
            JsonKind::String
        };
        if value.kind != expected_kind {
            return Err(ExternalEnvelopeJsonError::WrongFieldType(field_index as u8));
        }
        fields[field_index] = Some(value);

        parser.whitespace();
        match parser.input.get(parser.position) {
            Some(b',') => {
                parser.position += 1;
                parser.whitespace();
                if parser.input.get(parser.position) == Some(&b'}') {
                    return Err(ExternalEnvelopeJsonError::InvalidJson);
                }
            }
            Some(b'}') => {
                parser.position += 1;
                closed = true;
            }
            _ => return Err(ExternalEnvelopeJsonError::InvalidJson),
        }
    }

    parser.whitespace();
    if parser.position != input.len() {
        return Err(ExternalEnvelopeJsonError::InvalidJson);
    }

    let version = fields[0].ok_or(ExternalEnvelopeJsonError::MissingField(0))?;
    let algorithm = fields[1].ok_or(ExternalEnvelopeJsonError::MissingField(1))?;
    let nonce = fields[2].ok_or(ExternalEnvelopeJsonError::MissingField(2))?;
    let ciphertext = fields[3].ok_or(ExternalEnvelopeJsonError::MissingField(3))?;
    Ok(ExternalEnvelopeSpans {
        version,
        algorithm,
        nonce,
        ciphertext,
    })
}

/// Visits decoded JSON string content without allocating a `String`.
///
/// ASCII values are passed as `Some(byte)`; valid non-ASCII Unicode scalar values are passed as
/// `None`. The callback may return `false` to stop after a decision has become final. Callers
/// must pass a string span that already passed `Parser::string`.
pub(crate) fn visit_json_string_ascii(
    input: &[u8],
    span: ValueSpan,
    mut visitor: impl FnMut(Option<u8>) -> bool,
) -> bool {
    if span.kind != JsonKind::String || span.end < span.start + 2 {
        return false;
    }
    let Some(content) = input.get(span.start + 1..span.end - 1) else {
        return false;
    };
    let mut index = 0;
    while index < content.len() {
        let item = match content[index] {
            b'\\' => {
                index += 1;
                let Some(escape) = content.get(index).copied() else {
                    return false;
                };
                index += 1;
                match escape {
                    b'"' => Some(b'"'),
                    b'\\' => Some(b'\\'),
                    b'/' => Some(b'/'),
                    b'b' => Some(0x08),
                    b'f' => Some(0x0C),
                    b'n' => Some(b'\n'),
                    b'r' => Some(b'\r'),
                    b't' => Some(b'\t'),
                    b'u' => {
                        let Some(digits) = content.get(index..index + 4) else {
                            return false;
                        };
                        let Some(unit) = decode_hex_unit(digits) else {
                            return false;
                        };
                        index += 4;
                        if (0xD800..=0xDBFF).contains(&unit) {
                            let Some(low_prefix) = content.get(index..index + 2) else {
                                return false;
                            };
                            if low_prefix != b"\\u" {
                                return false;
                            }
                            let Some(low_digits) = content.get(index + 2..index + 6) else {
                                return false;
                            };
                            let Some(low) = decode_hex_unit(low_digits) else {
                                return false;
                            };
                            if !(0xDC00..=0xDFFF).contains(&low) {
                                return false;
                            }
                            index += 6;
                            None
                        } else if (0xDC00..=0xDFFF).contains(&unit) {
                            return false;
                        } else if unit <= 0x7F {
                            Some(unit as u8)
                        } else {
                            None
                        }
                    }
                    _ => return false,
                }
            }
            byte if byte.is_ascii() => {
                index += 1;
                Some(byte)
            }
            byte => {
                let width = match byte {
                    0xC2..=0xDF => 2,
                    0xE0..=0xEF => 3,
                    0xF0..=0xF4 => 4,
                    _ => return false,
                };
                index += width;
                None
            }
        };
        if !visitor(item) {
            return false;
        }
    }
    true
}

pub(crate) fn json_string_equals_ascii(input: &[u8], span: ValueSpan, expected: &str) -> bool {
    let mut position = 0;
    let mut matches = true;
    let completed = visit_json_string_ascii(input, span, |byte| {
        let Some(byte) = byte else {
            matches = false;
            return false;
        };
        if expected.as_bytes().get(position) != Some(&byte) {
            matches = false;
            return false;
        }
        position += 1;
        true
    });
    completed && matches && position == expected.len()
}

/// Re-reads a known schema object into a fixed-size set of expected member spans.
///
/// The complete document has already passed `StrictDocument::parse`, so unknown
/// values, duplicates, UTF-8, and depth were checked before this semantic pass.
pub(crate) fn schema_fields<const N: usize>(
    input: &[u8],
    object: ValueSpan,
    object_depth: usize,
    expected_names: [&'static str; N],
) -> Result<[Option<ValueSpan>; N], SchemaObjectError> {
    if object.kind != JsonKind::Object {
        return Err(SchemaObjectError::InvalidJson);
    }

    let mut parser = Parser {
        input,
        position: object.start,
        fields: RootFields::default(),
    };
    parser.position += 1;
    parser.whitespace();
    let mut fields = [None; N];
    if parser.input.get(parser.position) == Some(&b'}') {
        parser.position += 1;
        return if parser.position == object.end {
            Ok(fields)
        } else {
            Err(SchemaObjectError::InvalidJson)
        };
    }

    loop {
        parser.whitespace();
        let key_span = parser
            .string()
            .map_err(|_| SchemaObjectError::InvalidJson)?;
        let key = parser
            .decode_string(key_span)
            .map_err(|_| SchemaObjectError::InvalidJson)?;
        let index = expected_names
            .iter()
            .position(|expected| *expected == key)
            .ok_or(SchemaObjectError::ForbiddenField)?;

        parser.whitespace();
        if parser.input.get(parser.position) != Some(&b':') {
            return Err(SchemaObjectError::InvalidJson);
        }
        parser.position += 1;
        let value = parser
            .value(object_depth, false)
            .map_err(|_| SchemaObjectError::InvalidJson)?;
        if fields[index].replace(value).is_some() {
            return Err(SchemaObjectError::DuplicateProperty);
        }

        parser.whitespace();
        match parser.input.get(parser.position) {
            Some(b',') => parser.position += 1,
            Some(b'}') => {
                parser.position += 1;
                return if parser.position == object.end {
                    Ok(fields)
                } else {
                    Err(SchemaObjectError::InvalidJson)
                };
            }
            _ => return Err(SchemaObjectError::InvalidJson),
        }
    }
}

struct Parser<'a> {
    input: &'a [u8],
    position: usize,
    fields: RootFields,
}

fn decode_hex_unit(digits: &[u8]) -> Option<u16> {
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

impl Parser<'_> {
    fn whitespace(&mut self) {
        while matches!(
            self.input.get(self.position),
            Some(b' ' | b'\t' | b'\n' | b'\r')
        ) {
            self.position += 1;
        }
    }

    fn external_envelope_field_index(&self, span: ValueSpan) -> Option<usize> {
        const NAMES: [&str; 4] = ["version", "algorithm", "nonce", "ciphertext"];
        let mut candidates = [true; 4];
        let mut decoded_length = 0;
        let max_length = NAMES.iter().map(|name| name.len()).max().unwrap_or(0);
        let completed = visit_json_string_ascii(self.input, span, |byte| {
            let Some(byte) = byte else {
                candidates.fill(false);
                return false;
            };
            if decoded_length >= max_length {
                candidates.fill(false);
                return false;
            }
            for (index, name) in NAMES.iter().enumerate() {
                if candidates[index] && name.as_bytes().get(decoded_length) != Some(&byte) {
                    candidates[index] = false;
                }
            }
            decoded_length += 1;
            candidates.iter().any(|candidate| *candidate)
        });
        if !completed {
            return None;
        }
        NAMES.iter().enumerate().find_map(|(index, name)| {
            (candidates[index] && decoded_length == name.len()).then_some(index)
        })
    }

    fn scalar_or_container_kind(&mut self) -> Result<ValueSpan, JsonError> {
        self.whitespace();
        let start = self.position;
        match self.input.get(self.position) {
            Some(b'{') => Ok(ValueSpan {
                start,
                end: start + 1,
                kind: JsonKind::Object,
            }),
            Some(b'[') => Ok(ValueSpan {
                start,
                end: start + 1,
                kind: JsonKind::Array,
            }),
            _ => self.value(1, false),
        }
    }

    fn value(&mut self, parent_depth: usize, capture_root: bool) -> Result<ValueSpan, JsonError> {
        self.whitespace();
        let start = self.position;
        let kind = match self.input.get(self.position).copied() {
            Some(b'{') => {
                let depth = self.next_depth(parent_depth)?;
                self.object(depth, capture_root)?;
                JsonKind::Object
            }
            Some(b'[') => {
                let depth = self.next_depth(parent_depth)?;
                self.array(depth)?;
                JsonKind::Array
            }
            Some(b'"') => {
                self.string()?;
                JsonKind::String
            }
            Some(b't') => {
                self.literal(b"true")?;
                JsonKind::Boolean
            }
            Some(b'f') => {
                self.literal(b"false")?;
                JsonKind::Boolean
            }
            Some(b'n') => {
                self.literal(b"null")?;
                JsonKind::Null
            }
            Some(b'-' | b'0'..=b'9') => {
                self.number()?;
                JsonKind::Number
            }
            _ => return Err(JsonError::InvalidJson),
        };

        Ok(ValueSpan {
            start,
            end: self.position,
            kind,
        })
    }

    fn next_depth(&self, parent_depth: usize) -> Result<usize, JsonError> {
        let depth = parent_depth
            .checked_add(1)
            .ok_or(JsonError::DepthLimitExceeded)?;
        if depth > MAX_JSON_DEPTH {
            return Err(JsonError::DepthLimitExceeded);
        }
        Ok(depth)
    }

    fn object(&mut self, depth: usize, capture_root: bool) -> Result<(), JsonError> {
        self.position += 1;
        self.whitespace();
        if self.input.get(self.position) == Some(&b'}') {
            self.position += 1;
            return Ok(());
        }

        let mut keys = HashSet::new();
        loop {
            self.whitespace();
            let key_span = self.string()?;
            let key = self.decode_string(key_span)?;
            let root_key = if capture_root {
                match key.as_str() {
                    "formatVersion" => Some(RootKey::FormatVersion),
                    "vaultId" => Some(RootKey::VaultId),
                    "createdAt" => Some(RootKey::CreatedAt),
                    "kdf" => Some(RootKey::Kdf),
                    "wrappedRegistryKey" => Some(RootKey::WrappedRegistryKey),
                    _ => None,
                }
            } else {
                None
            };
            if !keys.insert(key) {
                return Err(JsonError::DuplicateProperty);
            }

            self.whitespace();
            if self.input.get(self.position) != Some(&b':') {
                return Err(JsonError::InvalidJson);
            }
            self.position += 1;

            let value = self.value(depth, false)?;
            if let Some(root_key) = root_key {
                match root_key {
                    RootKey::FormatVersion => self.fields.format_version = Some(value),
                    RootKey::VaultId => self.fields.vault_id = Some(value),
                    RootKey::CreatedAt => self.fields.created_at = Some(value),
                    RootKey::Kdf => self.fields.kdf = Some(value),
                    RootKey::WrappedRegistryKey => self.fields.wrapped_registry_key = Some(value),
                }
            }

            self.whitespace();
            match self.input.get(self.position) {
                Some(b',') => {
                    self.position += 1;
                    self.whitespace();
                    if self.input.get(self.position) == Some(&b'}') {
                        return Err(JsonError::InvalidJson);
                    }
                }
                Some(b'}') => {
                    self.position += 1;
                    return Ok(());
                }
                _ => return Err(JsonError::InvalidJson),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<(), JsonError> {
        self.position += 1;
        self.whitespace();
        if self.input.get(self.position) == Some(&b']') {
            self.position += 1;
            return Ok(());
        }

        loop {
            self.value(depth, false)?;
            self.whitespace();
            match self.input.get(self.position) {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(());
                }
                _ => return Err(JsonError::InvalidJson),
            }
        }
    }

    fn string(&mut self) -> Result<ValueSpan, JsonError> {
        let start = self.position;
        if self.input.get(self.position) != Some(&b'"') {
            return Err(JsonError::InvalidJson);
        }
        self.position += 1;
        loop {
            match self.input.get(self.position).copied() {
                Some(b'"') => {
                    self.position += 1;
                    return Ok(ValueSpan {
                        start,
                        end: self.position,
                        kind: JsonKind::String,
                    });
                }
                Some(b'\\') => {
                    self.position += 1;
                    match self.input.get(self.position).copied() {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                            self.position += 1;
                        }
                        Some(b'u') => {
                            let code_unit = self.unicode_unit()?;
                            if (0xD800..=0xDBFF).contains(&code_unit) {
                                if self
                                    .input
                                    .get(self.position..self.position.saturating_add(2))
                                    != Some(&b"\\u"[..])
                                {
                                    return Err(JsonError::InvalidJson);
                                }
                                self.position += 2;
                                let low = self.read_hex_unit()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(JsonError::InvalidJson);
                                }
                            } else if (0xDC00..=0xDFFF).contains(&code_unit) {
                                return Err(JsonError::InvalidJson);
                            }
                        }
                        _ => return Err(JsonError::InvalidJson),
                    }
                }
                Some(0x00..=0x1F) | None => return Err(JsonError::InvalidJson),
                Some(_) => self.position += 1,
            }
        }
    }

    fn decode_string(&self, span: ValueSpan) -> Result<String, JsonError> {
        let mut deserializer =
            serde_json::Deserializer::from_slice(&self.input[span.start..span.end]);
        let decoded = String::deserialize(&mut deserializer).map_err(|_| JsonError::InvalidJson)?;
        deserializer.end().map_err(|_| JsonError::InvalidJson)?;
        Ok(decoded)
    }

    fn unicode_unit(&mut self) -> Result<u16, JsonError> {
        if self.input.get(self.position) != Some(&b'u') {
            return Err(JsonError::InvalidJson);
        }
        self.position += 1;
        self.read_hex_unit()
    }

    fn read_hex_unit(&mut self) -> Result<u16, JsonError> {
        let end = self.position.checked_add(4).ok_or(JsonError::InvalidJson)?;
        let bytes = self
            .input
            .get(self.position..end)
            .ok_or(JsonError::InvalidJson)?;
        let mut value = 0_u16;
        for byte in bytes {
            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(JsonError::InvalidJson),
            };
            value = value * 16 + u16::from(digit);
        }
        self.position = end;
        Ok(value)
    }

    fn literal(&mut self, expected: &[u8]) -> Result<(), JsonError> {
        let end = self
            .position
            .checked_add(expected.len())
            .ok_or(JsonError::InvalidJson)?;
        if self.input.get(self.position..end) != Some(expected) {
            return Err(JsonError::InvalidJson);
        }
        self.position = end;
        Ok(())
    }

    fn number(&mut self) -> Result<(), JsonError> {
        if self.input.get(self.position) == Some(&b'-') {
            self.position += 1;
        }

        match self.input.get(self.position) {
            Some(b'0') => {
                self.position += 1;
                if matches!(self.input.get(self.position), Some(b'0'..=b'9')) {
                    return Err(JsonError::InvalidJson);
                }
            }
            Some(b'1'..=b'9') => {
                self.position += 1;
                while matches!(self.input.get(self.position), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(JsonError::InvalidJson),
        }

        if self.input.get(self.position) == Some(&b'.') {
            self.position += 1;
            let fraction_start = self.position;
            while matches!(self.input.get(self.position), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == fraction_start {
                return Err(JsonError::InvalidJson);
            }
        }

        if matches!(self.input.get(self.position), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.input.get(self.position), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let exponent_start = self.position;
            while matches!(self.input.get(self.position), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == exponent_start {
                return Err(JsonError::InvalidJson);
            }
        }

        Ok(())
    }
}
