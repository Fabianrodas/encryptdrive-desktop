use std::fmt;

/// A canonical lowercase UUID string did not match the 36-character hyphenated form.
///
/// The error intentionally omits the supplied text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifierParseError;

impl fmt::Display for IdentifierParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("identifier is not canonical lowercase UUID text")
    }
}

impl std::error::Error for IdentifierParseError {}

/// A canonical UUID identifying a user in VF1 metadata.
///
/// `UserId` cannot be substituted for a vault or manifest-entry identifier in typed APIs.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId([u8; 16]);

impl UserId {
    /// Parses exactly 36-character canonical lowercase UUID text.
    ///
    /// This validates text representation only. It does not constrain UUID version or variant
    /// bits and does not generate an identifier.
    pub fn from_canonical_str(text: &str) -> Result<Self, IdentifierParseError> {
        parse_canonical_uuid_bytes(text).map(Self)
    }

    /// Returns the 16 UUID bytes.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns canonical lowercase UUID text in a fixed-size byte array.
    pub fn to_canonical_bytes(self) -> [u8; 36] {
        format_uuid_bytes(&self.0)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_uuid(&self.0, formatter)
    }
}

impl fmt::Debug for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UserId")
            .field("redacted", &true)
            .finish()
    }
}

/// A canonical UUID identifying a manifest entry in VF1 metadata.
///
/// This is distinct from a content/blob identifier and is the correct file identifier for AAD.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryId([u8; 16]);

impl EntryId {
    /// Parses exactly 36-character canonical lowercase UUID text.
    ///
    /// This validates text representation only. It does not constrain UUID version or variant
    /// bits and does not generate an identifier.
    pub fn from_canonical_str(text: &str) -> Result<Self, IdentifierParseError> {
        parse_canonical_uuid_bytes(text).map(Self)
    }

    /// Returns the 16 UUID bytes.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns canonical lowercase UUID text in a fixed-size byte array.
    pub fn to_canonical_bytes(self) -> [u8; 36] {
        format_uuid_bytes(&self.0)
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_uuid(&self.0, formatter)
    }
}

impl fmt::Debug for EntryId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntryId")
            .field("redacted", &true)
            .finish()
    }
}

/// Parses the one canonical UUID text representation accepted by creation and typed APIs.
///
/// This intentionally does not impose version or variant bits. It is separate from any
/// historical metadata-reader policy.
pub(crate) fn parse_canonical_uuid_bytes(text: &str) -> Result<[u8; 16], IdentifierParseError> {
    let bytes = text.as_bytes();
    if bytes.len() != 36 || !bytes.is_ascii() {
        return Err(IdentifierParseError);
    }

    let mut decoded = [0_u8; 16];
    let mut output = 0;
    let mut high_nibble = None;
    for (position, byte) in bytes.iter().copied().enumerate() {
        if matches!(position, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return Err(IdentifierParseError);
            }
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            _ => return Err(IdentifierParseError),
        };
        if let Some(high) = high_nibble.take() {
            decoded[output] = (high << 4) | digit;
            output += 1;
        } else {
            high_nibble = Some(digit);
        }
    }
    if output != decoded.len() || high_nibble.is_some() {
        return Err(IdentifierParseError);
    }
    Ok(decoded)
}

pub(crate) fn format_uuid_bytes(bytes: &[u8; 16]) -> [u8; 36] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut formatted = [0_u8; 36];
    let mut out = 0;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            formatted[out] = b'-';
            out += 1;
        }
        formatted[out] = HEX[(byte >> 4) as usize];
        formatted[out + 1] = HEX[(byte & 0x0f) as usize];
        out += 2;
    }
    formatted
}

pub(crate) fn write_uuid(bytes: &[u8; 16], formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let text = format_uuid_bytes(bytes);
    // Every byte is an ASCII hex digit or hyphen, so this conversion is infallible.
    formatter.write_str(std::str::from_utf8(&text).expect("UUID formatter emits ASCII"))
}

#[cfg(test)]
mod tests {
    use super::{format_uuid_bytes, parse_canonical_uuid_bytes};

    #[test]
    fn canonical_byte_formatter_round_trips_shared_parser() {
        let text = "550e8400-e29b-41d4-a716-446655440000";
        let bytes = parse_canonical_uuid_bytes(text).unwrap();
        assert_eq!(format_uuid_bytes(&bytes), text.as_bytes());
    }
}
