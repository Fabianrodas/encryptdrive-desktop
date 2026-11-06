use std::io::{self, BufRead};
use vault_format::{
    VaultId, logical_name_eq_ignore_case, strip_java_whitespace, validate_logical_name,
};

fn main() -> Result<(), String> {
    let mut comparisons = 0_usize;
    let mut strips = 0_usize;
    let mut names = 0_usize;
    let mut identifiers = 0_usize;
    let mut saw_whitespace_vector = false;

    for (line_number, line) in io::stdin().lock().lines().enumerate() {
        let line = line.map_err(|error| format!("failed to read oracle vector: {error}"))?;
        let mut columns = line.split('|');
        match columns.next() {
            Some("META") => {
                if columns.count() != 3 {
                    return Err("malformed oracle metadata".to_owned());
                }
            }
            Some("W") => {
                let expected = columns.next().ok_or("missing whitespace vector")?;
                if columns.next().is_some() || saw_whitespace_vector {
                    return Err("malformed duplicate whitespace vector".to_owned());
                }
                saw_whitespace_vector = true;
                let mut observed = Vec::new();
                for code_point in 0..=0x10ffff_u32 {
                    if (0xd800..=0xdfff).contains(&code_point) {
                        continue;
                    }
                    let character = char::from_u32(code_point).expect("non-surrogate scalar");
                    let mut encoded = [0_u8; 4];
                    let scalar = character.encode_utf8(&mut encoded);
                    if strip_java_whitespace(scalar).is_empty() {
                        observed.push(format!("{code_point:x}"));
                    }
                }
                if observed.join(",") != expected {
                    return Err(format!(
                        "whitespace scalar set differs at input line {}",
                        line_number + 1
                    ));
                }
            }
            Some("S") => {
                let input = parse_scalars(columns.next().ok_or("missing strip input")?)?;
                let expected = parse_scalars(columns.next().ok_or("missing strip result")?)?;
                if columns.next().is_some() {
                    return Err("malformed strip vector".to_owned());
                }
                if strip_java_whitespace(&input) != expected {
                    return Err(format!(
                        "strip vector differs at input line {}",
                        line_number + 1
                    ));
                }
                strips += 1;
            }
            Some("Q") => {
                let left = parse_scalars(columns.next().ok_or("missing left string")?)?;
                let right = parse_scalars(columns.next().ok_or("missing right string")?)?;
                let expected = columns.next().ok_or("missing equality result")?;
                if columns.next().is_some() {
                    return Err("malformed equality vector".to_owned());
                }
                let expected = match expected {
                    "true" => true,
                    "false" => false,
                    _ => return Err("invalid equality result".to_owned()),
                };
                let observed = logical_name_eq_ignore_case(&left, &right);
                if observed != expected {
                    return Err(format!(
                        "equalsIgnoreCase vector differs at input line {}",
                        line_number + 1
                    ));
                }
                comparisons += 1;
            }
            Some("N") => {
                let input = parse_scalars(columns.next().ok_or("missing name input")?)?;
                let expected_status = columns.next().ok_or("missing name status")?;
                let expected_output = columns.next().ok_or("missing name result")?;
                if columns.next().is_some() {
                    return Err("malformed logical-name vector".to_owned());
                }
                match (expected_status, validate_logical_name(&input)) {
                    ("OK", Ok(actual)) if encode_scalars(actual) == expected_output => {}
                    ("ERR", Err(_)) => {}
                    _ => {
                        return Err(format!(
                            "logical-name vector differs at input line {}",
                            line_number + 1
                        ));
                    }
                }
                names += 1;
            }
            Some("U") => {
                let canonical = columns.next().ok_or("missing UUID text")?;
                if columns.next().is_some() {
                    return Err("malformed canonical UUID vector".to_owned());
                }
                let parsed = VaultId::from_canonical_str(canonical).map_err(|_| {
                    format!(
                        "Java canonical UUID rejected at input line {}",
                        line_number + 1
                    )
                })?;
                if parsed.to_string() != canonical {
                    return Err(format!(
                        "UUID canonical round trip differs at input line {}",
                        line_number + 1
                    ));
                }
                identifiers += 1;
            }
            Some("SUMMARY") => {
                // The summary is informational and independently checked by the calling script.
            }
            _ => {
                return Err(format!(
                    "unknown or empty oracle record at line {}",
                    line_number + 1
                ));
            }
        }
    }

    if !saw_whitespace_vector
        || strips < 7
        || names < 16
        || identifiers < 16
        || comparisons < 10_000
    {
        return Err("oracle corpus was incomplete".to_owned());
    }
    println!(
        "Java differential passed: 1,112,064 scalar whitespace checks; {strips} strip strings; {names} logical-name cases; {identifiers} UUID canonical forms; {comparisons} equalsIgnoreCase pairs."
    );
    Ok(())
}

fn encode_scalars(value: &str) -> String {
    value
        .chars()
        .map(|character| format!("{:x}", character as u32))
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_scalars(value: &str) -> Result<String, String> {
    let mut result = String::new();
    if value.is_empty() {
        return Ok(result);
    }
    for scalar in value.split(',') {
        let code_point = u32::from_str_radix(scalar, 16)
            .map_err(|_| "invalid hexadecimal code point in oracle vector".to_owned())?;
        let character = char::from_u32(code_point)
            .ok_or_else(|| "oracle vector contains a non-scalar value".to_owned())?;
        result.push(character);
    }
    Ok(result)
}
