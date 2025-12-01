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

    // Simple regex-like parsing manually
    let mut u = 1;
    let mut i = 1;
    
    // Extract numbers after u and i
    let u_idx = lower.find('u')?;
    let i_idx = lower.find('i')?;

    // Check if followed by digit
    if u_idx + 1 < lower.len() {
        u = lower[u_idx+1..u_idx+2].parse().ok()?;
    }
    if i_idx + 1 < lower.len() {
        i = lower[i_idx+1..i_idx+2].parse().ok()?;
    }

    Some((u.clamp(1, 3), i.clamp(1, 3)))
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
}
