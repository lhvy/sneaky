use std::str;

// pub(crate) fn rot(bytes: &mut [u8], n: u8) {
//     for byte in bytes {
//         *byte = byte.wrapping_add(n);
//     }
// }

pub(crate) fn string_rot(message: &mut str, n: u8) {
    alphabetic_rot(unsafe { message.as_bytes_mut() }, n);
    debug_assert!(str::from_utf8(message.as_bytes()).is_ok());
}

pub(crate) fn alphabetic_rot(bytes: &mut [u8], n: u8) {
    for byte in bytes {
        if byte.is_ascii_alphabetic() {
            if byte.is_ascii_uppercase() {
                *byte = (*byte - b'A' + n) % 26 + b'A';
            } else {
                *byte = (*byte - b'a' + n) % 26 + b'a';
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cipher::string_rot;

    fn check(message: &str, output: &str, n: u8) {
        let mut message_copy = message.to_string();
        string_rot(&mut message_copy, n);
        assert_eq!(output, &message_copy);
        string_rot(&mut message_copy, 26 - n);
        assert_eq!(message, &message_copy);
    }

    #[test]
    fn empty_string() {
        check("", "", 1);
    }

    #[test]
    fn full_rotate() {
        check("this is a 26 rotate", "this is a 26 rotate", 26);
    }

    #[test]
    fn rotate() {
        check(
            "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM",
            "djreglhvbcnfqstuwxymkpioazDJREGLHVBCNFQSTUWXYMKPIOAZ",
            13,
        );
    }

    #[test]
    fn special_chars() {
        check("1234567890!@#$%^&*()_+-=", "1234567890!@#$%^&*()_+-=", 1)
    }
}
