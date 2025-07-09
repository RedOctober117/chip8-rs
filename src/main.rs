use crate::chip8::Chip8;

pub mod chip8;

pub fn main() {
    let mut emulator = Chip8::new();
    emulator.load_program(std::fs::read("IBM Logo.ch8").unwrap());
    emulator.fetch_decode_execute();
}
