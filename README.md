# sneaky

### What?

A general purpose steganography tool written in Rust (🦀). This is an education tool I used to learn more about steganography, it has support for image encoding with LSB, my own somewhat cursed encoding in WAV files, and a very cool method of encoding data inside of executable files (only Mach-O ARM64 for now).

The image LSB feature is able to store arbitrary data in an RGB image, along with encryption/randomisation of the data using a user-inputted password to mitigate against steganalysis and brute-forcing. On top of this, the LSB encoding/decoding process was rewritten several times and been benchmarked against a [popular steganography library](https://github.com/ragibson/Steganography) that uses NumPy (a highly optimised C library). This Rust tool is significantly faster for both encoding and decoding. Note, you can run the benchmarks yourself by running `cargo run --bin bench --release`.

### Where?

You can find the latest releases for Windows, Linux and macOS (universal binary) [here](https://nightly.link/lhvy/sneaky/workflows/cd.yaml/master). Don't forget to `chmod +x` on macOS or Linux!

If you want to test out the tool, try completing some of the basic activities [here](https://gist.github.com/lhvy/69411bd76ad555f89238d17d1291d79a).

If you want to try reading/implementing some super basic image LSB code in Python, check out [this](https://gist.github.com/lhvy/09104a92da7c74aaf238494f3ee3a739).

### How?

The entire tool is operated by an interactive terminal interface. Just run the binary to start the tool, and follow the instructions. All files outputted will be stored in the current working directory.

### Stretch Goals

On top of the existing functionality, these are potential items to improve in the future. If anyone else has suggestions (or wants to add code 😳), please feel free to make an issue or PR.

- [x] Interactive UI
- [x] LSB benchmarks
- [x] Basic tests
- [x] Sample activities
- [ ] More error checking
- [ ] x86 binary support
- [ ] Chaining inputs without having to restart the program
- [ ] Automation
- [ ] Testing of unsafe code
- [ ] Phase Encoding or DWT for audio
