use crate::chip8::Chip8;

pub mod chip8;

const ROM: &str = "Tetris [Fran Dachille, 1991].ch8";

pub fn main() {
    // println!(
    //     "{:?}",
    //     u8::from_str_radix(&char::from_u32(97).unwrap().to_string(), 16)
    // );
    let mut emulator = Chip8::new();
    emulator.load_program(std::fs::read(ROM).unwrap());
    emulator.fetch_decode_execute();
}
