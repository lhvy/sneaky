pub fn from_bytes(bytes: &[u8]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

    for byte in bytes {
        for i in 0..8 {
            let bit = (byte >> (7 - i)) & 1;
            for i in (1..i16::MAX).step_by(64) {
                match bit {
                    0 => {
                        writer.write_sample(-i).unwrap();
                        writer.write_sample(i).unwrap();
                    }
                    1 => {
                        writer.write_sample(1).unwrap();
                        writer.write_sample(1).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
            for _ in 0..1 {
                match bit {
                    0 => {
                        writer.write_sample(i16::MIN).unwrap();
                        writer.write_sample(i16::MAX).unwrap();
                    }
                    1 => {
                        writer.write_sample(0).unwrap();
                        writer.write_sample(0).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
            for i in (1..i16::MAX).rev().step_by(64) {
                match bit {
                    0 => {
                        writer.write_sample(-i).unwrap();
                        writer.write_sample(i).unwrap();
                    }
                    1 => {
                        writer.write_sample(1).unwrap();
                        writer.write_sample(1).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub fn to_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut reader = hound::WavReader::new(bytes).unwrap();

    let mut bytes = Vec::new();
    let mut byte = 0;
    let mut bits_read = 0;
    // Loop over sets of 4 samples and detect 0 or 1
    for chunk in reader
        .samples::<i16>()
        .collect::<Result<Vec<i16>, _>>()
        .unwrap()
        .chunks(2)
    {
        match chunk {
            [i16::MIN, i16::MAX] => {
                byte <<= 1;
                bits_read += 1;
            }
            [0, 0] => {
                byte = (byte << 1) | 1;
                bits_read += 1;
            }
            _ => {}
        }

        if bits_read == 8 {
            bytes.push(byte);
            byte = 0;
            bits_read = 0;
        }
    }

    bytes
}
