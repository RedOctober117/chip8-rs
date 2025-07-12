extern crate sdl2;

use crate::{chip8::Chip8, renderer::Renderer};
pub mod chip8;
pub mod renderer;

const ROM: &str = "roms/demos/Zero Demo [zeroZshadow, 2007].ch8";

const PIXEL_SIZE: u32 = 8;

pub fn main() {
    let mut emulator = Chip8::new();
    emulator.load_program(std::fs::read(ROM).unwrap());

    let mut renderer = Renderer::init(emulator, 1280, 720, PIXEL_SIZE);
    renderer.cycle();
}
