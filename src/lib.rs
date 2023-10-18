pub fn lsb_encode(message: String, mut image: image::RgbImage, n_bits: u8) {
    // Convert the message to a vector of bytes
    let mut message = message.as_bytes().to_vec();
    // Add another byte for optimisation purposes (required or data will be lost 💀)
    message.push(0);

    // Ensure that the message will fit in the image
    let max_bytes = (image.width() * image.height() * 3) / (8 / n_bits) as u32 - 4;
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
    let len = u32::try_from(payload.len() - 1).unwrap();

    // Write the length of the message to the LSB of the first 32 bytes
    for (i, byte) in carrier[..32].iter_mut().enumerate() {
        let bit = (len >> i) & 1;
        *byte = (*byte & 0xFE) | bit as u8;
    }

    let len = len as usize;

    let mut payload_bit_index: usize = 0;
    for carrier_byte in &mut carrier[32..] {
        if payload_bit_index >= len * 8 {
            break;
        }

        let payload_byte_index = payload_bit_index / 8;
        let offset = payload_bit_index % 8;
        let payload_bytes = unsafe {
            (*payload.get_unchecked(payload_byte_index) as usize) << 8
                | (*payload.get_unchecked(payload_byte_index + 1) as usize)
        };

        let mask = (1 << n_bits) - 1;
        let shift_amount = 16 - n_bits - offset;

        let payload_bits = (payload_bytes >> shift_amount) & mask;
        payload_bit_index += n_bits;

        let carrier_bits = *carrier_byte as usize & (0xFF << n_bits);
        *carrier_byte = (carrier_bits | payload_bits) as u8;
    }
}

pub fn lsb_decode(image: image::RgbImage, n_bits: u8) {
    // Decode the message from the image
    let message = lsb_raw_decode(image.as_flat_samples().as_slice(), n_bits as usize);

    // Convert the message to a string
    let message = String::from_utf8(message).unwrap();
    println!("Message: {}", message);
}

pub fn lsb_raw_decode(carrier: &[u8], n_bits: usize) -> Vec<u8> {
    // Read the length of the message from the LSB of the first 32 bytes
    let mut len = 0;
    for (i, byte) in carrier[..32].iter().enumerate() {
        let bit = (*byte & 1) as usize;
        len |= bit << i;
    }

    // Decode the message from the image
    let mut payload = Vec::with_capacity(len);
    let mut byte = 0;
    let mut bits_read = 0;

    for carrier_byte in &carrier[32..] {
        // Extract n_bits from the image, but don't go past the end of the byte
        let bits_to_extract = n_bits.min(8 - bits_read);

        // Create a mask to extract the bits
        let mask = (1_usize << bits_to_extract) - 1;
        let extracted_bits = (*carrier_byte as usize) >> (n_bits - bits_to_extract) & mask;

        // Write the bits to the message
        byte = (byte << bits_to_extract) | extracted_bits;
        bits_read += bits_to_extract;

        if bits_read == 8 {
            payload.push(byte as u8);
            if payload.len() == len {
                break;
            }
            byte = 0;
            bits_read = 0;
        }

        // Read the remaining bits for the next byte
        if bits_to_extract < n_bits {
            bits_read = n_bits - bits_to_extract;
            let mask = (1_usize << bits_read) - 1;
            byte = (*carrier_byte as usize) & mask;
        }
    }

    payload
}
