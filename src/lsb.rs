use image::ImageBuffer;
use rand::seq::SliceRandom;
use rand::SeedableRng;

pub fn encode(message: String, mut image: image::RgbImage, n_bits: u8, key: Option<u64>) {
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

    let flat_image = &mut image.as_flat_samples_mut();
    let carrier = flat_image.as_mut_slice();

    match key {
        Some(key) => {
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(key);

            let mut index_map = (0..carrier.len()).collect::<Vec<_>>();
            index_map.shuffle(&mut rng);

            let mut shuffled_carrier = Vec::with_capacity(carrier.len());
            for i in &index_map {
                shuffled_carrier.push(carrier[*i]);
            }

            raw_encode(&message, &mut shuffled_carrier, n_bits as usize);

            for (i, shuffled_carrier) in index_map.into_iter().zip(shuffled_carrier) {
                carrier[i] = shuffled_carrier;
            }
        }
        None => {
            raw_encode(&message, carrier, n_bits as usize);
        }
    }

    // Save the image
    image.save("out.png").unwrap();
    println!("Saved image to ./out.png");
}

pub fn raw_encode(payload: &[u8], carrier: &mut [u8], n_bits: usize) {
    let len = u32::try_from(payload.len() - 1).unwrap();

    // Write the length of the message to the LSB of the first 32 bytes
    for (i, byte) in carrier[..32].iter_mut().enumerate() {
        let bit = (len >> i) & 1;
        *byte = (*byte & 0xFE) | bit as u8;
    }

    let len = len as usize;

    if n_bits == 8 && len <= carrier.len() - 32 {
        carrier[32..32 + len].copy_from_slice(&payload[..len]);
        return;
    }

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

pub fn decode(image: image::RgbImage, n_bits: u8, key: Option<u64>) {
    // Decode the message from the image
    let message = match key {
        Some(key) => {
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(key);

            let flat_image = image.as_flat_samples();
            let mut carrier = flat_image.as_slice().to_vec();

            carrier.shuffle(&mut rng);

            raw_decode(&carrier, n_bits as usize)
        }
        None => raw_decode(image.as_flat_samples().as_slice(), n_bits as usize),
    };

    // Convert the message to a string
    let message = String::from_utf8(message).unwrap();
    println!("Message: {}", message);
}

pub fn raw_decode(carrier: &[u8], n_bits: usize) -> Vec<u8> {
    // Read the length of the message from the LSB of the first 32 bytes
    let mut len = 0;
    for (i, byte) in carrier[..32].iter().enumerate() {
        let bit = (*byte & 1) as usize;
        len |= bit << i;
    }

    if n_bits == 8 {
        return carrier[32..32 + len].to_vec();
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

pub fn extract_bit_planes(image: image::RgbImage) {
    let mut planes = vec![ImageBuffer::new(image.width(), image.height()); 24].into_boxed_slice();
    let mut plane_pixels = planes
        .iter_mut()
        .map(|p| p.pixels_mut())
        .collect::<Vec<_>>();
    for src in image.pixels() {
        for channel in 0..3 {
            for bit in 0..8 {
                let dst = plane_pixels[channel * 8 + bit].next().unwrap();
                *dst = if src[channel] & (1 << bit) == 0 {
                    image::Luma([255_u8])
                } else {
                    image::Luma([0])
                };
            }
        }
    }

    // Save into output folder, labelled R, G, B 0-7
    // Folder first
    std::fs::create_dir_all("output").unwrap();
    // Then save the images
    for (i, plane) in planes.iter().enumerate() {
        let color = match i / 8 {
            0 => "R",
            1 => "G",
            2 => "B",
            _ => unreachable!(),
        };
        plane
            .save(format!("output/{}{}.png", color, i % 8))
            .unwrap();
    }
    println!("Saved images to ./output");
}
