mod table {
    include!("java_case_fold_table.rs");
}

/// Returns Java 21's locale-independent simple case comparison key for one Unicode scalar.
///
/// The compact table is generated from the observable Java 21 API expression
/// `Character.toLowerCase(Character.toUpperCase(codePoint))`; no JDK or third-party source table
/// is copied. Its Unicode data basis is Unicode 15.0 as used by Java SE 21.
pub(crate) fn java_case_key(code_point: u32) -> u32 {
    match table::JAVA_CASE_FOLD_MAP.binary_search_by_key(&code_point, |(source, _)| *source) {
        Ok(index) => table::JAVA_CASE_FOLD_MAP[index].1,
        Err(_) => code_point,
    }
}

#[cfg(test)]
mod tests {
    use super::java_case_key;
    use crate::identifiers::parse_canonical_uuid_bytes;

    #[test]
    fn generated_map_is_sorted_unique_and_canonical() {
        assert_eq!(super::table::JAVA_CASE_FOLD_MAP.len(), 1_456);
        let mut previous = None;
        for &(source, target) in super::table::JAVA_CASE_FOLD_MAP {
            assert!(char::from_u32(source).is_some());
            assert!(char::from_u32(target).is_some());
            assert_ne!(source, target);
            assert!(previous.is_none_or(|value| value < source));
            previous = Some(source);
        }
    }

    #[test]
    fn generated_map_matches_the_jdk_21_api_derived_checksum() {
        let mut checksum = 0xcbf29ce484222325_u64;
        for &(source, target) in super::table::JAVA_CASE_FOLD_MAP {
            for value in [source, target] {
                for byte in value.to_le_bytes() {
                    checksum ^= u64::from(byte);
                    checksum = checksum.wrapping_mul(0x100000001b3);
                }
            }
        }
        assert_eq!(checksum, 0x42556af8aefc14ab);
    }

    #[test]
    fn case_key_does_not_affect_identifier_parser_behavior() {
        assert!(parse_canonical_uuid_bytes("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert_ne!(java_case_key('A' as u32), 'A' as u32);
    }
}
