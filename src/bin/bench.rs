use rand::RngCore;

fn main() {
    let mut payload = vec![0u8; 1000 * 1000];
    let mut carrier = vec![0u8; 10 * 1000 * 1000];
    rand::thread_rng().fill_bytes(&mut payload);
    rand::thread_rng().fill_bytes(&mut carrier);
    // Optimisation for the encoder
    payload.push(0);

    const TESTS: usize = 500;

    println!("Encoding speed (MB/s):");
    for n in 1..=8 {
        let time = std::time::Instant::now();
        for _ in 0..TESTS {
            #[allow(clippy::unit_arg)]
            std::hint::black_box(sneaky::lsb::raw_encode(&payload, &mut carrier, n));
        }
        let elapsed = time.elapsed().as_secs_f64();
        let processed = payload.len() as f64 * TESTS as f64 / (1000.0 * 1000.0);
        println!("{} bits: {:.2} MB/s", n, processed / elapsed)
    }

    println!("Decoding speed (MB/s):");
    for n in 1..=8 {
        let time = std::time::Instant::now();
        for _ in 0..TESTS {
            std::hint::black_box(sneaky::lsb::raw_decode(&carrier, n));
        }
        let elapsed = time.elapsed().as_secs_f64();
        let processed = payload.len() as f64 * TESTS as f64 / (1000.0 * 1000.0);
        println!("{} bits: {:.2} MB/s", n, processed / elapsed)
    }
}
