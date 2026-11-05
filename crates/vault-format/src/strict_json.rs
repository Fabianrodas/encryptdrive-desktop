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

impl Parser<'_> {
    fn whitespace(&mut self) {
        while matches!(
            self.input.get(self.position),
            Some(b' ' | b'\t' | b'\n' | b'\r')
        ) {
            self.position += 1;
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
