pub fn lsb_encode(message: String, mut image: image::RgbImage, n_bits: u8) {
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

    lsb_raw_encode(&message, image.as_flat_samples_mut().as_mut_slice(), n_bits);

    // Save the image
    image.save("out.png").unwrap();
    println!("Saved image to ./out.png");
}

pub fn lsb_raw_encode(payload: &[u8], carrier: &mut [u8], n_bits: u8) {
    let mut input_bits_index: usize = 0;
    for carrier_byte in carrier {
        if input_bits_index >= payload.len() * 8 {
            break;
        }

        // Extract n_bits from the carrier byte
        let mut payload_bits = 0;
        for k in (0..n_bits).rev() {
            if input_bits_index >= payload.len() * 8 {
                break;
            }
            let byte_index = input_bits_index / 8;
            let offset = input_bits_index % 8;
            let msg_bit = (payload[byte_index] >> (7 - offset)) & 1;
            payload_bits |= msg_bit << k;
            input_bits_index += 1;
        }

        let carrier_bits = *carrier_byte & (0xFF << n_bits);
        *carrier_byte = carrier_bits | payload_bits;
    }
}

pub fn lsb_decode(image: image::RgbImage, n_bits: u8) {
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
