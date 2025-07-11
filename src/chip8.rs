use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::Renderer;

// use tokio::sync::mpsc::{Receiver, Sender};

const HALF_BYTE: u32 = 4;
const FULL_BYTE: u32 = 8;

pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;
const START_ADDRESS: u16 = 0x200;

const FONTSET_START_ADDRESS: u8 = 0x50;

const DELAY: u64 = 6;

pub enum Interrupts {
    Keyboard(u8),
}

pub struct Chip8 {
    mem: [u8; 4000],
    display: [u8; VIDEO_HEIGHT * VIDEO_WIDTH],
    pc: u16,
    index_register: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
    keypad: [u8; 16],
    renderer: Renderer,
}

impl Chip8 {
    pub fn load_program(&mut self, program: Vec<u8>) {
        let mut memory_index = START_ADDRESS as usize;

        for byte in program {
            self.mem[memory_index] = byte;
            memory_index += 1;
        }
    }

    pub fn new() -> Self {
        Self {
            mem: [0_u8; 4000],
            display: [0; VIDEO_HEIGHT * VIDEO_WIDTH],
            pc: START_ADDRESS,
            index_register: 0,
            stack: vec![],
            delay_timer: 60,
            sound_timer: 60,
            registers: [0; 16],
            keypad: [0; 16],
            renderer: Renderer::init(),
        }
    }

    pub fn get_display(&self) -> &[u8; VIDEO_HEIGHT * VIDEO_WIDTH] {
        &self.display
    }

    pub fn fetch_decode_execute(&mut self) {
        loop {
            let opcode = u16::from(self.mem[self.pc as usize]).wrapping_shl(FULL_BYTE)
                | self.mem[self.pc as usize + 1] as u16;

            let instruction_prefix: u8 =
                (opcode & 0xF000).wrapping_shr(FULL_BYTE + HALF_BYTE) as u8;
            let nibble_0: u8 = (opcode & 0x0F00).wrapping_shr(FULL_BYTE) as u8;
            let nibble_1: u8 = (opcode & 0x00F0).wrapping_shr(HALF_BYTE) as u8;
            let nibble_2: u8 = (opcode & 0x000F) as u8;

            let nnn: u16 = u16::from(nibble_0).wrapping_shl(FULL_BYTE)
                | nibble_1.wrapping_shl(HALF_BYTE) as u16
                | nibble_2 as u16;
            let kk: u8 = nibble_1.wrapping_shl(HALF_BYTE) | nibble_2;

            self.pc += 2;

            match (instruction_prefix, nibble_0, nibble_1, nibble_2) {
                (0, 0, 0xE, 0) => self.x00e0(),
                (0, 0, 0xE, 0xE) => self.x00ee(),
                (1, _, _, _) => self.x1nnn(nnn),
                (2, _, _, _) => self.x2nnn(nnn),
                (6, _, _, _) => self.x6xkk(nibble_0, kk),
                (7, _, _, _) => self.x7xkk(nibble_0, kk),
                (8, _, _, 0) => self.x8xy0(nibble_0, nibble_1),
                (8, _, _, 1) => self.x8xy1(nibble_0, nibble_1),
                (8, _, _, 2) => self.x8xy2(nibble_0, nibble_1),
                (8, _, _, 3) => self.x8xy3(nibble_0, nibble_1),
                (8, _, _, 4) => self.x8xy4(nibble_0, nibble_1),
                (8, _, _, 5) => self.x8xy5(nibble_0, nibble_1),
                (8, _, _, 6) => self.x8xy6(nibble_0, nibble_1),
                (8, _, _, 7) => self.x8xy7(nibble_0, nibble_1),
                (8, _, _, 0xE) => self.x8xye(nibble_0, nibble_1),
                (3, _, _, _) => self.x3xkk(nibble_0, kk),
                (4, _, _, _) => self.x4xkk(nibble_0, kk),
                (5, _, _, 0) => self.x5xy0(nibble_0, nibble_1),
                (9, _, _, 0) => self.x9xy0(nibble_0, nibble_1),
                (0xA, _, _, _) => self.xannn(nnn),
                (0xB, _, _, _) => self.xbnnn(nnn),
                (0xC, _, _, _) => self.xcxkk(nibble_0, kk),
                (0xD, _, _, _) => self.xdxyn(nibble_0, nibble_1, nibble_2),
                (0xE, _, 9, 0xE) => self.xex9e(nibble_0),
                (0xE, _, 0xA, 1) => self.xexa1(nibble_0),
                (0xF, _, 1, 5) => self.xfx15(nibble_0),
                (0xF, _, 0, 7) => self.xfx07(nibble_0),
                (0xF, _, 1, 8) => self.xfx18(nibble_0),
                (0xF, _, 1, 0xE) => self.xfx1e(nibble_0),
                (0xF, _, 0, 0xA) => self.xfx0a(nibble_0),
                (0xF, _, 5, 5) => self.xfx55(nibble_0),
                (0xF, _, 6, 5) => self.xfx65(nibble_0),
                (0xF, _, 3, 3) => self.xfx33(nibble_0),
                (0xF, _, 2, 9) => self.xfx29(nibble_0),
                _ => todo!(
                    "{:?} {:?} {:?} {:?} NOT YET IMPLEMENTED!",
                    char::from_digit(instruction_prefix as u32, 16),
                    char::from_digit(nibble_0 as u32, 16),
                    char::from_digit(nibble_1 as u32, 16),
                    char::from_digit(nibble_2 as u32, 16),
                ),
            }

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
            println!("{:?}", self.keypad);
            self.renderer.draw(&self.display, &mut self.keypad);

            sleep(Duration::from_millis(DELAY));
        }
    }

    // Return from subroutine
    pub fn x00ee(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    // Execute subroutine at addr nnn
    pub fn x2nnn(&mut self, nnn: u16) {
        // self.stack.insert(self.stack_pointer as usize, self.pc);
        self.stack.push(self.pc);
        self.pc = nnn;
    }

    // clear display
    pub fn x00e0(&mut self) {
        self.display = [0; VIDEO_WIDTH * VIDEO_HEIGHT];
    }

    // JMP nnn
    pub fn x1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    // LD vx, kk
    pub fn x6xkk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    // ADD vx, kk, overflow wraps
    pub fn x7xkk(&mut self, x: u8, kk: u8) {
        let (sum, _) = self.registers[x as usize].overflowing_add(kk);
        self.registers[x as usize] = sum;
    }

    // LD vx, vy
    pub fn x8xy0(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    }

    // ADD vx, vy, vf = carry
    pub fn x8xy4(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[0xF] = 0;

        let (sum, overflowed) = vx.overflowing_add(vy);

        self.registers[x as usize] = sum;

        if overflowed {
            self.registers[0xF] = 1;
        }
    }

    // SUB vx, vy, vf = !borrow
    pub fn x8xy5(&mut self, x: u8, y: u8) {
        self.registers[0xF] = 1;
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (difference, overflowed) = vx.overflowing_sub(vy);
        self.registers[x as usize] = difference;

        if overflowed {
            self.registers[0xF] = 0;
        }
    }

    // SUB vy, vx, vf = !borrow
    pub fn x8xy7(&mut self, x: u8, y: u8) {
        self.registers[0xF] = 1;
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (difference, overflowed) = vy.overflowing_sub(vx);
        self.registers[x as usize] = difference;

        if overflowed {
            self.registers[0xF] = 0;
        }
    }

    // AND vx, vy
    pub fn x8xy2(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vx & vy;
    }

    // OR vx, vy
    pub fn x8xy1(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vx | vy;
    }

    // XOR vx, vy
    pub fn x8xy3(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vx ^ vy;
    }

    // LD i, nnn
    pub fn xannn(&mut self, nnn: u16) {
        self.index_register = nnn;
    }

    // SHR vx, vy, 1; vf = LSB
    pub fn x8xy6(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];

        self.registers[0xF] = 0x01 & vy;
        self.registers[x as usize] = vy >> 1;
    }

    // SHL vx, vy, 1; vf = MSB
    pub fn x8xye(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];

        self.registers[0xF] = (0x80 & vy).reverse_bits();
        self.registers[x as usize] = vy << 1;
    }

    // RAND vx, kk
    pub fn xcxkk(&mut self, x: u8, kk: u8) {
        let nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u8;

        self.registers[x as usize] = nanoseconds & kk;
    }

    // JMP nnn + v0
    pub fn xbnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.registers[0x0] as u16;
    }

    // SKPE vx, nn
    pub fn x3xkk(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.pc += 2;
        }
    }

    // SKPE vx, nn
    pub fn x5xy0(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        if vx == vy {
            self.pc += 2;
        }
    }

    // SKPNE vx, nn
    pub fn x4xkk(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.pc += 2;
        }
    }

    // SKPNE vx, vy
    pub fn x9xy0(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        if vx != vy {
            self.pc += 2;
        }
    }

    // LD delay, vx
    pub fn xfx15(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.delay_timer = vx;
    }

    // LD vx, delay
    pub fn xfx07(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    }

    // LD sound, vx
    pub fn xfx18(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.sound_timer = vx;
    }

    // LD vx, input
    pub fn xfx0a(&mut self, x: u8) {
        let mut found = false;
        for index in 0..self.keypad.len() {
            if self.keypad[index] != 0 {
                self.registers[x as usize] = self.keypad[index];
                found = true;
            }
        }

        if !found {
            self.pc -= 2;
        }
    }

    // ADD i, vx
    pub fn xfx1e(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.index_register += vx as u16;
    }

    // SKPE vx, key
    pub fn xex9e(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        if self.keypad[vx as usize] != 0 {
            self.pc += 2;
        }
    }

    // SKPNE vx, key
    pub fn xexa1(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        if self.keypad[vx as usize] == 0 {
            self.pc += 2;
        }
    }

    // LD i-vx, v0-vx
    pub fn xfx55(&mut self, x: u8) {
        for index in 0..=x {
            self.mem[self.index_register as usize + index as usize] =
                self.registers[index as usize];
        }

        self.index_register += x as u16 + 1; // ?
    }

    // LD decimal values of vx to i-i2
    pub fn xfx33(&mut self, x: u8) {
        let mut vx = self.registers[x as usize];
        // 253
        self.mem[self.index_register as usize + 2] = vx % 10;
        vx /= 10;
        self.mem[self.index_register as usize + 1] = vx % 10;
        vx /= 10;
        self.mem[self.index_register as usize] = vx % 10;
    }

    // LDv0-vx, i-vx
    pub fn xfx65(&mut self, x: u8) {
        for index in 0..=x {
            self.registers[index as usize] =
                self.mem[self.index_register as usize + index as usize];
        }

        self.index_register += x as u16 + 1; // ?
    }

    // LD i, mem address f hex digit sprite
    pub fn xfx29(&mut self, x: u8) {
        let vx = self.registers[x as usize];

        self.index_register = (FONTSET_START_ADDRESS + (5 * vx)) as u16;
    }

    // print screen
    pub fn xdxyn(&mut self, x: u8, y: u8, n: u8) {
        let x_pos = self.registers[x as usize] as usize % VIDEO_WIDTH;
        let y_pos = self.registers[y as usize] as usize % VIDEO_HEIGHT;
        let height = n;

        self.registers[0xF] = 0;

        for row in 0..height as usize {
            // one byte represents 8 columns of pixels, hence the shifting below
            // ie, byte 7 represents 0000111 pixels
            let sprite_byte = self.mem[self.index_register as usize + row as usize];
            for col in 0..8 {
                let sprite_bit = sprite_byte & (0x80_u8 >> col);

                let current_display_pointer =
                    &mut self.display[(y_pos + row) * VIDEO_WIDTH + (x_pos + col)];

                let current_display_is_on = current_display_pointer.to_owned() != 0;

                if sprite_bit != 0 {
                    *current_display_pointer ^= 0xFF;
                }

                if current_display_is_on {
                    self.registers[0xF] = 1;
                }
            }
        }
    }
}
