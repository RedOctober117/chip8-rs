extern crate sdl2;

use crate::{
    chip8::{Chip8, VIDEO_HEIGHT, VIDEO_WIDTH},
    renderer::Renderer,
};
pub mod chip8;
pub mod renderer;

// const ROM: &str = "Tetris [Fran Dachille, 1991].ch8";
// const ROM: &str = "IBM Logo.ch8";
const ROM: &str = "5-quirks.ch8";
// const ROM: &str = "SCTEST.CH8";
// const ROM: &str = "test_opcode.ch8";
// const ROM: &str = "Tank.ch8";

const PIXEL_SIZE: u32 = 8;

pub fn main() {
    let mut emulator = Chip8::new();
    emulator.load_program(std::fs::read(ROM).unwrap());
    emulator.fetch_decode_execute(Renderer::init(
        PIXEL_SIZE,
        VIDEO_WIDTH as u32,
        VIDEO_HEIGHT as u32,
    ));
}
