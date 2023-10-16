use rand::RngCore;
use sneaky::lsb_raw_encode;

fn main() {
    let mut payload = vec![0u8; 1000 * 1000];
    let mut carrier = vec![0u8; 10 * 1000 * 1000];
    rand::thread_rng().fill_bytes(&mut payload);
    rand::thread_rng().fill_bytes(&mut carrier);

    const TESTS: usize = 500;

    for n in 0..=8 {
        let time = std::time::Instant::now();
        for _ in 0..TESTS {
            lsb_raw_encode(&payload, &mut carrier, n);
        }
        let elapsed = time.elapsed().as_secs_f64();
        let processed = payload.len() as f64 * TESTS as f64 / (1000.0 * 1000.0);
        println!("{} bits: {:.2} MB/s", n, processed / elapsed)
    }
}
