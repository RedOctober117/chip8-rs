use crate::chip8::Chip8;

pub mod chip8;

// const ROM: &str = "Tetris [Fran Dachille, 1991].ch8";
// const ROM: &str = "test_opcode.ch8";
const ROM: &str = "Tank.ch8";

pub fn main() {
    // println!("{}", 0x80_u8.rotate_left(1));
    // println!("{}", 0x80_u8.rotate_right(1));
    // println!("{}", 0x80_u8 << 1);
    // println!("{}", 0x80_u8 >> 1);
    // let (interrupt_upstream, interrupt_downstream) = tokio::sync::mpsc::channel(1);

    let mut emulator = Chip8::new();
    emulator.load_program(std::fs::read(ROM).unwrap());
    emulator.fetch_decode_execute();
}
