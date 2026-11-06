use std::fmt;

use crate::java_case_fold::java_case_key;

/// A sanitized error produced by creation/rename-compatible logical-name validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalNameError {
    /// The name is empty after Java-compatible edge stripping.
    Empty,
    /// The name is exactly `.`.
    Dot,
    /// The name is exactly `..`.
    DotDot,
    /// The name contains U+0000.
    ContainsNul,
    /// The name exceeds 255 Unicode code points after stripping.
    TooLong,
}

impl fmt::Display for LogicalNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "logical name is empty",
            Self::Dot => "logical name cannot be '.'",
            Self::DotDot => "logical name cannot be '..'",
            Self::ContainsNul => "logical name contains a NUL character",
            Self::TooLong => "logical name exceeds 255 Unicode code points",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for LogicalNameError {}

/// Removes exactly the edge characters recognized by Java 21 `Character.isWhitespace`.
///
/// This is intentionally different from Rust `str::trim`: Java retains NBSP (U+00A0), figure
/// space (U+2007), narrow no-break space (U+202F), and NEL (U+0085), and strips U+001C–U+001F.
/// The returned slice borrows the input and performs no allocation or Unicode normalization.
pub fn strip_java_whitespace(value: &str) -> &str {
    value.trim_matches(java_is_whitespace)
}

/// Validates a creation/rename logical name and returns the Java-stripped borrowed value.
///
/// This does not validate historical manifest data and does not imply filesystem path or target
/// filename safety. Slash, backslash, colon, Windows reserved names, other controls and trailing
/// periods remain permissible. Unicode is not normalized.
pub fn validate_logical_name(value: &str) -> Result<&str, LogicalNameError> {
    let candidate = strip_java_whitespace(value);
    if candidate.is_empty() {
        return Err(LogicalNameError::Empty);
    }
    if candidate == "." {
        return Err(LogicalNameError::Dot);
    }
    if candidate == ".." {
        return Err(LogicalNameError::DotDot);
    }
    if candidate.contains('\0') {
        return Err(LogicalNameError::ContainsNul);
    }
    if candidate.chars().take(256).count() > 255 {
        return Err(LogicalNameError::TooLong);
    }
    Ok(candidate)
}

/// Compares two valid UTF-8 strings using Java 21 `String.equalsIgnoreCase` scalar semantics.
///
/// Java first requires equal UTF-16 code-unit lengths. Each scalar comparison then applies
/// `lower(upper(codePoint))`, using the versioned Unicode 15.0 mapping generated from the Java 21
/// `Character` API. This does not normalize Unicode and does not perform multi-character folding.
pub fn logical_name_eq_ignore_case(left: &str, right: &str) -> bool {
    if left.encode_utf16().count() != right.encode_utf16().count() {
        return false;
    }

    let mut left_chars = left.chars();
    let mut right_chars = right.chars();
    loop {
        match (left_chars.next(), right_chars.next()) {
            (Some(left), Some(right)) => {
                if java_case_key(left as u32) != java_case_key(right as u32) {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn java_is_whitespace(character: char) -> bool {
    matches!(
        character as u32,
        0x0009..=0x000d
            | 0x001c..=0x0020
            | 0x1680
            | 0x2000..=0x2006
            | 0x2008..=0x200a
            | 0x2028..=0x2029
            | 0x205f
            | 0x3000
    )
}
