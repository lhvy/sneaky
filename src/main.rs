use std::path::Path;

fn main() {
    // Input a message using Inquire
    let message = inquire::Text::new("Enter a message").prompt().unwrap();

    // Input an image using Inquire
    let image_path = inquire::Text::new("Enter an image path")
        .with_validator(|a: &str| {
            if !Path::new(a).exists() {
                return Ok(inquire::validator::Validation::Invalid(
                    "File does not exist".into(),
                ));
            } else {
                return Ok(inquire::validator::Validation::Valid);
            }
        })
        .prompt()
        .unwrap();

    let image = image::open(image_path).unwrap().to_rgb8();

    // How many bits to use for the message, 1-8
    let n_bits = inquire::Text::new("How many bits to use for LSB?")
        .with_validator(|a: &str| {
            if a.parse::<u8>().unwrap() > 8 {
                return Ok(inquire::validator::Validation::Invalid(
                    "Must be between 1 and 8".into(),
                ));
            } else {
                return Ok(inquire::validator::Validation::Valid);
            }
        })
        .prompt()
        .unwrap()
        .parse::<u8>()
        .unwrap();

    lsb_encode(message, image, n_bits);

    lsb_decode(n_bits);
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
    }

    // Convert the message to a vector of bits
    let mut message_bits = Vec::new();
    for byte in message {
        for i in 0..8 {
            message_bits.push((byte >> (7 - i)) & 1);
        }
    }

    // Write the message to the image
    let mut i = 0;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel_mut(x, y);
            for j in 0..3 {
                if i == message_bits.len() {
                    break;
                }

                // Extract n_bits from the message
                let mut byte = 0;
                for _ in 0..n_bits {
                    if i == message_bits.len() {
                        break;
                    }

                    byte = (byte << 1) | message_bits[i];
                    i += 1;
                }

                // Write the bits to the image
                pixel[j] = (pixel[j] & (0xFF << n_bits)) | byte;
            }
        }
    }

    // Save the image
    image.save("out.png").unwrap();
}

fn lsb_decode(n_bits: u8) {
    let image = image::open("out.png").unwrap().to_rgb8();

    // Decode the message from the image
    let mut message = Vec::new();
    let mut byte = 0;
    let mut bits_read = 0;
    let mut done = false;
    for y in 0..image.height() {
        for x in 0..image.width() {
            // Extract bits from the image, stop when a null byte is found
            // 1 pixel may not contain the entire byte, so we need to extract multiple pixels
            let pixel = image.get_pixel(x, y);

            for j in 0..3 {
                if done {
                    break;
                }

                // Extract n_bits from the image, but don't go past the end of the byte
                let mut bits_to_extract = n_bits;
                if bits_to_extract > 8 - bits_read {
                    bits_to_extract = 8 - bits_read;
                }

                // Create a mask to extract the bits
                let mask = (1 << bits_to_extract) - 1;
                let extracted_bits = pixel[j] >> (n_bits - bits_to_extract) & mask;

                // Write the bits to the message
                byte = (byte << bits_to_extract) | extracted_bits;
                bits_read += bits_to_extract;

                if bits_read == 8 {
                    message.push(byte);
                    if byte == 0 {
                        done = true;
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
