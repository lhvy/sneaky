pub(crate) fn rot(message: String, n: u8) -> String {
    message
        .chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            }

            let base = if c.is_ascii_uppercase() {
                'A' as u8
            } else {
                'a' as u8
            };

            ((c as u8 - base + n) % 26 + base) as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::cipher::rot;

    #[test]
    fn empty_string() {
        assert_eq!(rot("".to_string(), 1), "".to_string())
    }

    #[test]
    fn full_rotate() {
        assert_eq!(
            rot("this is a 26 rotate".to_string(), 26),
            "this is a 26 rotate".to_string()
        );
    }

    #[test]
    fn rotate() {
        assert_eq!(
            rot(
                "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM".to_string(),
                13
            ),
            "djreglhvbcnfqstuwxymkpioazDJREGLHVBCNFQSTUWXYMKPIOAZ".to_string()
        );
    }

    #[test]
    fn special_chars() {
        assert_eq!(
            rot("1234567890!@#$%^&*()_+-=".to_string(), 13),
            "1234567890!@#$%^&*()_+-=".to_string()
        );
    }
}
