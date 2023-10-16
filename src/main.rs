mod cipher;

use inquire::validator::Validation;
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
            let image_path = get_path();
            let image = image::open(image_path).unwrap().to_rgb8();
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

fn lsb_encode(message: String, mut image: image::RgbImage, n_bits: u8) {
    // Convert the message to a vector of bytes
    let mut message = message.as_bytes().to_vec();
    // Add a null byte to the end of the message to signify the end of the message.
    message.push(0);

    // Ensure that the message will fit in the image
    let max_bytes = (image.width() * image.height() * 3) / (8 / n_bits) as u32;
    if message.len() > max_bytes as usize {
        println!(
            "Message is too long to fit in the image. Maximum message length is {} bytes, including the null byte",
            max_bytes
        );
        return;
    }

    // Write the message to the image
    let mut i = 0;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel_mut(x, y);
            for j in 0..3 {
                if i >= message.len() * 8 {
                    break;
                }

                // Extract n_bits from the message
                let mut byte = 0;
                for k in (0..n_bits).rev() {
                    if i >= message.len() * 8 {
                        break;
                    }
                    let byte_index = i / 8;
                    let offset = i % 8;
                    let msg_bit = (message[byte_index] >> (7 - offset)) & 1;
                    byte |= msg_bit << k;
                    i += 1;
                }

                // Write the bits to the image
                pixel[j] = (pixel[j] & (0xFF << n_bits)) | byte;
            }
        }
    }

    // Save the image
    image.save("out.png").unwrap();
    println!("Saved image to ./out.png");
}

fn lsb_decode(image: image::RgbImage, n_bits: u8) {
    // Decode the message from the image
    let mut message = Vec::new();
    let mut byte = 0;
    let mut bits_read = 0;
    'decode: for y in 0..image.height() {
        for x in 0..image.width() {
            // Extract bits from the image, stop when a null byte is found
            // 1 pixel may not contain the entire byte, so we need to extract multiple pixels
            let pixel = image.get_pixel(x, y);

            for j in 0..3 {
                // Extract n_bits from the image, but don't go past the end of the byte
                let bits_to_extract = n_bits.min(8 - bits_read);

                // Create a mask to extract the bits
                let mask = (1 << bits_to_extract) - 1;
                let extracted_bits = pixel[j] >> (n_bits - bits_to_extract) & mask;

                // Write the bits to the message
                byte = (byte << bits_to_extract) | extracted_bits;
                bits_read += bits_to_extract;

                if bits_read == 8 {
                    message.push(byte);
                    if byte == 0 {
                        break 'decode;
                    }
                    byte = 0;
                    bits_read = 0;
                }

                // Read the remaining bits for the next byte
                if bits_to_extract < n_bits {
                    bits_read = n_bits - bits_to_extract;
                    let mask = (1 << bits_read) - 1;
                    byte = pixel[j] & mask;
                }
            }
        }
    }

    // Convert the message to a string
    let message = String::from_utf8(message).unwrap();
    println!("Message: {}", message);
}
