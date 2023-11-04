#![allow(clippy::unusual_byte_groupings)]

use goblin::Object;
use std::fs;

struct Biterator<'a> {
    bytes: &'a [u8],
    bit: u8,
}

impl<'a> Biterator<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, bit: 8 }
    }
}

impl Iterator for Biterator<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        if self.bit == 0 {
            self.bit = 8;
            self.bytes = &self.bytes[1..];
        }
        self.bit -= 1;

        if self.bytes.is_empty() {
            return None;
        }

        Some(self.bytes[0] & (1 << self.bit) != 0)
    }
}

#[derive(Default)]
struct Binit {
    bytes: Vec<u8>,
    bit: u8,
}

impl Binit {
    fn add(&mut self, bit: bool) {
        if self.bit == 0 {
            self.bit = 8;
            self.bytes.push(0);
        }
        self.bit -= 1;

        if bit {
            *self.bytes.last_mut().unwrap() |= 1 << self.bit;
        }
    }

    fn complete(&self) -> Option<&[u8]> {
        // if self.bit == 0 {
        //     Some(&self.bytes)
        // } else {
        //     None
        // }
        Some(&self.bytes)
    }

    fn len(&mut self) -> Option<u32> {
        if self.bytes.len() == 4 && self.bit == 0 {
            // Cast first 4 bytes to u32 and then remove them
            let len = u32::from_le_bytes(self.bytes[..4].try_into().unwrap());
            self.bytes = self.bytes[4..].to_vec();
            Some(len)
        } else {
            None
        }
    }
}

fn winnable_instruction(i: u32) -> bool {
    // ADD/ADDS (shifted register)
    // X_0_X_01011_00_0_MMMMM_000000_NNNNN_XXXXX
    let add_mask = 0b0_1_0_11111_11_1_00000_111111_00000_00000;
    let add = 0b0_0_0_01011_00_0_00000_000000_00000_00000;
    if (i & add_mask) == add {
        return true;
    }

    // ADC/ADCS
    // X_0_X_11010000_MMMMM_000000_NNNNN_XXXXX
    let adc_mask = 0b0_1_0_11111111_00000_111111_00000_00000;
    let adc = 0b0_0_0_11010000_00000_000000_00000_00000;
    if (i & adc_mask) == adc {
        return true;
    }

    // AND/ANDS (shifted register)
    // X_XX_01010_00_0_MMMMM_000000_NNNNN_XXXXX
    let and_mask = 0b0_00_11111_11_1_00000_111111_00000_00000;
    let and = 0b0_00_01010_00_0_00000_000000_00000_00000;
    if (i & and_mask) == and {
        return true;
    }

    // ORR (shifted register) - technically also MOV (register) 😳
    // X_01_01010_00_0_MMMMM_000000_NNNNN_XXXXX
    let orr_mask = 0b0_11_11111_11_1_00000_111111_00000_00000;
    let orr = 0b0_01_01010_00_0_00000_000000_00000_00000;
    if (i & orr_mask) == orr {
        return true;
    }

    // EOR (shifted register)
    // X_10_01010_00_0_MMMMM_000000_NNNNN_XXXXX
    let eor_mask = 0b0_11_11111_11_1_00000_111111_00000_00000;
    let eor = 0b0_10_01010_00_0_00000_000000_00000_00000;
    if (i & eor_mask) == eor {
        return true;
    }

    false
}

pub fn inject_string(binary_path: &str, string_to_inject: &[u8], new_binary_path: &str) {
    let buffer = fs::read(binary_path).unwrap();

    // Add len as 4 bytes to the start of string to inject
    let len = string_to_inject.len() as u32;
    let len_bytes = len.to_le_bytes();
    let mut payload = len_bytes.to_vec();
    payload.extend_from_slice(string_to_inject);

    let mut biterator = Biterator::new(&payload);

    let object = Object::parse(&buffer).unwrap();
    match &object {
        Object::Elf(_) => todo!(),
        Object::PE(_) => todo!(),
        Object::Mach(mach) => match mach {
            goblin::mach::Mach::Fat(_) => todo!(),
            goblin::mach::Mach::Binary(binary) => {
                // Find the __TEXT segment
                let text_segment = binary
                    .segments
                    .iter()
                    .find(|&segment| segment.name().unwrap() == "__TEXT")
                    .unwrap_or_else(|| binary.segments.first().unwrap());

                // Find the __text section
                let text_section = text_segment
                    .sections()
                    .unwrap()
                    .iter()
                    .find(|&section| section.0.name().unwrap() == "__text")
                    .unwrap()
                    .1;

                let text_section = unsafe {
                    std::slice::from_raw_parts_mut(
                        text_section.as_ptr().cast::<u32>().cast_mut(),
                        text_section.len() / 4,
                    )
                };

                for instruction_mut in text_section {
                    let instruction = *instruction_mut;
                    if !winnable_instruction(instruction) {
                        continue;
                    }

                    // Extract Rm and Rn
                    let rm_mask = 0b0_0_0_00000_00_0_11111_000000_00000_00000;
                    let rm = (instruction & rm_mask) >> 16;
                    let rn_mask = 0b0_0_0_00000_00_0_00000_000000_11111_00000;
                    let rn = (instruction & rn_mask) >> 5;

                    // If rm > rn, swap rm and rn in the instruction

                    if rm == rn {
                        continue;
                    }

                    match biterator.next() {
                        // true -> bigger, smaller
                        Some(true) if rm < rn => {
                            *instruction_mut =
                                instruction & !rm_mask & !rn_mask | (rn << 16) | (rm << 5)
                        }
                        // false -> smaller, bigger
                        Some(false) if rm > rn => {
                            *instruction_mut =
                                instruction & !rm_mask & !rn_mask | (rn << 16) | (rm << 5)
                        }
                        // None => break,
                        _ => {}
                    }
                }
            }
        },
        Object::Archive(_) => todo!(),
        Object::Unknown(_) => todo!(),
    }

    // Write the modified binary to a new file
    std::fs::write(new_binary_path, &buffer).unwrap();

    if let Object::Mach(_) = object {
        std::process::Command::new("codesign")
            .arg("--remove-signature")
            .arg(new_binary_path)
            .status()
            .unwrap();

        std::process::Command::new("codesign")
            .arg("-s")
            .arg("-")
            .arg(new_binary_path)
            .status()
            .unwrap();
    }

    if biterator.next().is_some() {
        println!("String too long");
    }
}

pub fn extract_string(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut binit = Binit::default();

    let object = Object::parse(bytes).unwrap();
    match &object {
        Object::Elf(_) => todo!(),
        Object::PE(_) => todo!(),
        Object::Mach(mach) => match mach {
            goblin::mach::Mach::Fat(_) => todo!(),
            goblin::mach::Mach::Binary(binary) => {
                // Find the __TEXT segment
                let text_segment = binary
                    .segments
                    .iter()
                    .find(|&segment| segment.name().unwrap() == "__TEXT")
                    .unwrap_or_else(|| binary.segments.first().unwrap());

                // Find the __text section
                let text_section = text_segment
                    .sections()
                    .unwrap()
                    .iter()
                    .find(|&section| section.0.name().unwrap() == "__text")
                    .unwrap()
                    .1;

                let text_section = unsafe {
                    std::slice::from_raw_parts_mut(
                        text_section.as_ptr().cast::<u32>().cast_mut(),
                        text_section.len() / 4,
                    )
                };

                let mut len = None;
                let mut read = 0;

                for instruction_mut in text_section {
                    if len.is_none() {
                        if let Some(l) = binit.len() {
                            len = Some(l);
                            read = 0;
                        }
                    } else if read == len.unwrap() * 8 {
                        break;
                    }

                    let instruction = *instruction_mut;
                    if !winnable_instruction(instruction) {
                        continue;
                    }

                    // Extract Rm and Rn
                    let rm_mask = 0b0_0_0_00000_00_0_11111_000000_00000_00000;
                    let rm = (instruction & rm_mask) >> 16;
                    let rn_mask = 0b0_0_0_00000_00_0_00000_000000_11111_00000;
                    let rn = (instruction & rn_mask) >> 5;
                    if rm == rn {
                        continue;
                    }

                    binit.add(rm > rn);

                    read += 1;
                }
            }
        },
        Object::Archive(_) => todo!(),
        Object::Unknown(_) => todo!(),
    }

    Some(binit.complete().unwrap().to_vec())
}

// Biterator tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biterator_populated() {
        let bytes = [0b01010101, 0b10101010];

        let mut biterator = Biterator::new(&bytes);
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));

        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));

        assert_eq!(biterator.next(), None);
    }

    #[test]
    fn pog() {
        let mut biterator = Biterator::new(b"pog");
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(false));

        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));

        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(false));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));
        assert_eq!(biterator.next(), Some(true));

        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);

        assert_eq!(biterator.next(), None);
    }

    #[test]
    fn biterator_empty() {
        let mut biterator = Biterator::new(&[]);
        assert_eq!(biterator.next(), None);
        assert_eq!(biterator.next(), None);
    }
}
