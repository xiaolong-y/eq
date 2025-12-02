pub fn parse_priority(input: &str) -> Option<(u8, u8)> {
    let mut urgency = 0;
    let mut importance = 0;

    // Check for shorthand notation (e.g., u2i3, i3u1)
    if let Some((u, i)) = parse_shorthand(input) {
        return Some((u, i));
    }

    // Check for symbol notation (e.g., !!$$)
    for c in input.chars() {
        match c {
            '!' => urgency += 1,
            '$' => importance += 1,
            _ => return None, // If contains other chars, it's not a priority string
        }
    }

    if urgency > 0 || importance > 0 {
        // Default to 1 if not specified but the other is
        let u = if urgency == 0 { 1 } else { urgency };
        let i = if importance == 0 { 1 } else { importance };
        Some((u.clamp(1, 3), i.clamp(1, 3)))
    } else {
        None
    }
}

fn parse_shorthand(input: &str) -> Option<(u8, u8)> {
    let lower = input.to_lowercase();
    if !lower.contains('u') || !lower.contains('i') {
        return None;
    }

    let mut u: Option<u8> = None;
    let mut i: Option<u8> = None;
    
    // Find 'u' followed by a digit
    if let Some(u_idx) = lower.find('u') {
        if u_idx + 1 < lower.len() {
            let next_char = lower.chars().nth(u_idx + 1)?;
            if next_char.is_ascii_digit() {
                u = next_char.to_digit(10).map(|d| d as u8);
            }
        }
    }
    
    // Find 'i' followed by a digit
    if let Some(i_idx) = lower.find('i') {
        if i_idx + 1 < lower.len() {
            let next_char = lower.chars().nth(i_idx + 1)?;
            if next_char.is_ascii_digit() {
                i = next_char.to_digit(10).map(|d| d as u8);
            }
        }
    }

    // Both must be found with valid digits
    match (u, i) {
        (Some(urgency), Some(importance)) => {
            Some((urgency.clamp(1, 3), importance.clamp(1, 3)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_parsing() {
        assert_eq!(parse_priority("!!!$$$"), Some((3, 3)));
        assert_eq!(parse_priority("!$"), Some((1, 1)));
        assert_eq!(parse_priority("!!"), Some((2, 1))); // Default importance 1
        assert_eq!(parse_priority("$$"), Some((1, 2))); // Default urgency 1
    }

    #[test]
    fn test_shorthand_parsing() {
        assert_eq!(parse_priority("u3i3"), Some((3, 3)));
        assert_eq!(parse_priority("i2u1"), Some((1, 2)));
        assert_eq!(parse_priority("u2i2"), Some((2, 2)));
    }
    
    #[test]
    fn test_invalid() {
        assert_eq!(parse_priority("abc"), None);
        assert_eq!(parse_priority("task!"), None); // Contains letters
    }

    #[test]
    fn test_edge_cases() {
        // Fix #2: These should not crash
        assert_eq!(parse_priority("ui"), None);
        assert_eq!(parse_priority("iu"), None);
        assert_eq!(parse_priority("u"), None);
        assert_eq!(parse_priority("i"), None);
        assert_eq!(parse_priority(""), None);
    }
}
