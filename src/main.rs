use inquire::validator::Validation;
use sneaky::{binary, cipher, lsb, wav};
use std::ops::BitXor;
use std::path::{Path, PathBuf};

fn main() {
    let options = vec!["Encode a message", "Decode a message", "Analyse a file"];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Encode a message" => encode(),
        "Decode a message" => decode(),
        "Analyse a file" => analyse(),
        _ => panic!("Invalid option"),
    }

    // Ask for confirmation before closing
    inquire::Confirm::new("Press enter to exit")
        .with_default(true)
        .prompt()
        .unwrap();
}

fn encode() {
    let mut message = inquire::Text::new("Enter a message").prompt().unwrap();

    let options = vec![
        "Encode using LSB in an image",
        "Encode using FSK to a wav",
        "Encode using a ROT cipher",
        "Encode into a Mach-O binary",
    ];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Encode using LSB in an image" => {
            let image_path = get_path();
            let image = image::open(image_path).unwrap().to_rgb8();
            let n_bits = get_bits();

            lsb::encode(message, image, n_bits, gen_seed());
        }
        "Encode using FSK to a wav" => {
            wav::from_bytes(message.as_bytes());
        }
        "Encode using a ROT cipher" => {
            // input any integer
            let n = get_string_rot_n();
            cipher::string_rot(&mut message, n);
            println!("Message: {}", message);
        }
        "Encode into a Mach-O binary" => {
            let path = get_path();
            binary::inject_string(
                path.to_str().unwrap(),
                message.as_bytes(),
                &format!("{}-out", path.display()),
            );
            println!("Saved to {}-out", path.display());
        }
        _ => panic!("Invalid option"),
    }
}

fn decode() {
    let options = vec!["Decode text", "Decode a file"];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    let mut bytes = match choice {
        "Decode text" => inquire::Text::new("Enter a message")
            .prompt()
            .unwrap()
            .into_bytes(),
        "Decode a file" => {
            let path = get_path();
            std::fs::read(path).unwrap()
        }
        _ => panic!("Invalid option"),
    };

    let options = vec![
        "Decode as an image using LSB",
        "Decode using FSK from a wav",
        "Decode using a ROT cipher",
        "Decode from a Mach-O binary",
    ];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Decode as an image using LSB" => {
            let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
            let n_bits = get_bits();
            lsb::decode(image, n_bits, gen_seed());
        }
        "Decode using FSK from a wav" => {
            let bytes = wav::to_bytes(&bytes);
            println!("Message: {}", String::from_utf8_lossy(&bytes));
        }
        "Decode using a ROT cipher" => {
            let n = get_string_rot_n();
            cipher::alphabetic_rot(&mut bytes, 26 - n);
            println!("Message: {}", String::from_utf8_lossy(&bytes));
        }
        "Decode from a Mach-O binary" => {
            let bytes = binary::extract_string(&bytes);
            println!(
                "Message: {}",
                String::from_utf8_lossy(bytes.unwrap().as_slice())
            );
        }
        _ => panic!("Invalid option"),
    }
}

fn analyse() {
    let options = vec!["Analyse an image's bit planes"];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Analyse an image's bit planes" => {
            let image_path = get_path();
            let image = image::open(image_path).unwrap().to_rgb8();
            lsb::extract_bit_planes(image);
        }
        _ => panic!("Invalid option"),
    }
}

fn get_bits() -> u8 {
    // How many bits to use for the message, 1-8
    let n_bits = inquire::Text::new("How many bits to use for LSB?")
        .with_validator(|a: &str| {
            if a.parse::<u8>().unwrap() > 8 {
                Ok(Validation::Invalid("Must be between 1 and 8".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
        .unwrap()
        .parse::<u8>()
        .unwrap();

    n_bits
}

fn get_path() -> PathBuf {
    let image_path = inquire::Text::new("Enter a file path")
        .with_validator(|a: &str| {
            if !Path::new(a.trim()).exists() {
                Ok(Validation::Invalid("File does not exist".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
        .unwrap();

    PathBuf::from(image_path.trim())
}

fn get_password() -> Option<String> {
    // Would the user like to use a password?
    let password = inquire::Confirm::new("Would you like to use a password?")
        .with_default(false)
        .prompt()
        .unwrap();

    if password {
        let password = inquire::Text::new("Enter a password").prompt().unwrap();
        Some(password)
    } else {
        None
    }
}

fn gen_seed() -> Option<u64> {
    let password = get_password()?;

    // https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html
    const K: u64 = 0x517cc1b727220a95;
    let mut hash: u64 = 0;
    for b in password.as_bytes() {
        hash = hash.rotate_left(5).bitxor(*b as u64).wrapping_mul(K);
    }

    Some(hash)
}

fn get_string_rot_n() -> u8 {
    inquire::Text::new("How many characters to rotate by?")
        .with_validator(|a: &str| {
            if let Ok(n) = a.parse::<u8>() {
                if n > 26 {
                    Ok(Validation::Invalid("Must be between 0 and 26".into()))
                } else {
                    Ok(Validation::Valid)
                }
            } else {
                Ok(Validation::Invalid("Must be an integer".into()))
            }
        })
        .prompt()
        .unwrap()
        .parse::<u8>()
        .unwrap()
}
