mod cipher;

use inquire::validator::Validation;
use sneaky::{lsb_decode, lsb_encode};
use std::path::{Path, PathBuf};

fn main() {
    let options = vec!["Encode a message", "Decode a message"];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Encode a message" => encode(),
        "Decode a message" => decode(),
        _ => panic!("Invalid option"),
    }
}

fn encode() {
    let mut message = inquire::Text::new("Enter a message").prompt().unwrap();

    let options = vec!["Encode using LSB in an image", "Encode using a ROT cipher"];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Encode using LSB in an image" => {
            let image_path = get_path();
            let image = image::open(image_path).unwrap().to_rgb8();
            let n_bits = get_bits();
            lsb_encode(message, image, n_bits);
        }
        "Encode using a ROT cipher" => {
            // input any integer
            let n = get_string_rot_n();
            cipher::string_rot(&mut message, n);
            println!("Message: {}", message);
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

    let options = vec!["Decode using LSB in an image", "Decode using a ROT cipher"];
    let choice = inquire::Select::new("Select an option", options)
        .prompt()
        .unwrap();

    match choice {
        "Decode using LSB in an image" => {
            let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
            let n_bits = get_bits();
            lsb_decode(image, n_bits);
        }
        "Decode using a ROT cipher" => {
            let n = get_string_rot_n();
            cipher::alphabetic_rot(&mut bytes, 26 - n);
            println!("Message: {}", String::from_utf8_lossy(&bytes));
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
    let image_path = inquire::Text::new("Enter an image path")
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
