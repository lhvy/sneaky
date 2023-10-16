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

    lsb_raw_encode(
        &message,
        image.as_flat_samples_mut().as_mut_slice(),
        n_bits as usize,
    );

    // Save the image
    image.save("out.png").unwrap();
    println!("Saved image to ./out.png");
}

pub fn lsb_raw_encode(payload: &[u8], carrier: &mut [u8], n_bits: usize) {
    let mut payload_bit_index: usize = 0;
    for carrier_byte in carrier {
        if payload_bit_index >= payload.len() * 8 {
            break;
        }

        let payload_byte_index = payload_bit_index / 8;
        let offset = payload_bit_index % 8;
        let mut payload_bytes: u16 =
            (unsafe { *payload.get_unchecked(payload_byte_index) } as u16) << 8;
        if (payload_byte_index + 1) < payload.len() {
            payload_bytes |= unsafe { *payload.get_unchecked(payload_byte_index + 1) } as u16;
        }

        let mask = (1 << n_bits) - 1;
        let shift_amount = 16 - n_bits - offset;

        let payload_bits = (payload_bytes >> shift_amount) as u8 & mask;
        payload_bit_index += n_bits;

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
