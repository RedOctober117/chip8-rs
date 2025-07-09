use std::{collections::HashMap, io::Write};

const HALF_BYTE: u32 = 4;
const FULL_BYTE: u32 = 8;

const VIDEO_WIDTH: usize = 64;
const VIDEO_HEIGHT: usize = 32;
const START_ADDRESS: u16 = 0x200;

pub struct Chip8 {
    mem: [u8; 4000],
    display: [u8; VIDEO_HEIGHT * VIDEO_WIDTH],
    pc: u16,
    index_register: u16,
    stack_pointer: u8,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
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
            stack_pointer: 0,
            stack: vec![],
            delay_timer: 60,
            sound_timer: 60,
            registers: [0; 16],
        }
    }

    pub fn fetch_decode_execute(mut self) {
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
                // clear screen
                (0, 0, 0xE, 0) => {
                    // println!("clear screen");
                    self.x00e0();
                }
                (0, 0, 0xE, 0xE) => self.x00ee(),
                // 1nnn: JP addr
                (1, _, _, _) => {
                    // println!("jump");
                    self.x1nnn(nnn);
                }
                (2, _, _, _) => self.x2nnn(nnn),
                // 6xkk: LD x, byte
                (6, _, _, _) => {
                    // println!("set vx");
                    self.x6xkk(nibble_0, kk);
                }

                // 7xkk, ADD x, byte
                (7, _, _, _) => {
                    // println!("set nn to vx");
                    self.x7xkk(nibble_0, kk);
                }
                // 8xy0: LD x, y
                (8, _, _, 0) => {
                    // println!("load y into x");
                    self.x8xy0(nibble_0, nibble_1);
                }
                (8, _, _, 1) => self.x8xy1(nibble_0, nibble_1),
                (8, _, _, 2) => self.x8xy2(nibble_0, nibble_1),
                (8, _, _, 3) => self.x8xy3(nibble_0, nibble_1),
                (8, _, _, 4) => self.x8xy4(nibble_0, nibble_1),
                (8, _, _, 5) => self.x8xy5(nibble_0, nibble_1),
                (8, _, _, 6) => self.x8xy6(nibble_0, nibble_1),
                (8, _, _, 7) => self.x8xy7(nibble_0, nibble_1),
                (8, _, _, 0xE) => self.x8xye(nibble_0, nibble_1),

                (3, _, _, _) => self.x3xnn(nibble_0, kk),
                (4, _, _, _) => self.x4xnn(nibble_0, kk),
                (5, _, _, 0) => self.x5xy0(nibble_0, nibble_1),
                (9, _, _, 0) => self.x9xy0(nibble_0, nibble_1),

                // Annn: LD I, addr
                (0xA, _, _, _) =>
                // println!("set index register");
                {
                    self.xannn(nnn);
                }
                (0xB, _, _, _) => self.xbnnn(nnn),
                (0xC, _, _, _) => self.xcxnn(),

                // Dxyn: DRW x, y, nibble
                (0xD, _, _, _) => self.xdxyn(nibble_0, nibble_1, nibble_2),
                (0xE, _, 9, 0xE) => self.xex9e(),
                (0xF, _, 1, 5) => self.xfx15(nibble_0),
                (0xF, _, 0, 7) => self.xfx07(nibble_0),
                (0xF, _, 1, 8) => self.xfx18(nibble_0),
                (0xF, _, 0, 0xA) => self.xfx0a(nibble_0),

                // _ => println!("IMPLEMENT {}{}{}{}", nib_0, nib_1, nib_2, nib_3),
                _ => todo!(),
            }
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }
    }

    pub fn x00ee(&mut self) {
        self.stack_pointer -= 1;
        self.pc = self.stack.get(self.stack_pointer as usize).unwrap().clone() as u16;
    }

    pub fn x2nnn(&mut self, nnn: u16) {
        self.stack.insert(self.stack_pointer as usize, self.pc);
        self.stack_pointer += 1;
        self.pc = nnn;
    }

    pub fn x00e0(&mut self) {
        self.display = [0; VIDEO_WIDTH * VIDEO_HEIGHT];
    }

    pub fn x1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    pub fn x6xkk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    pub fn x7xkk(&mut self, x: u8, kk: u8) {
        let (sum, _) = self.registers[x as usize].overflowing_add(kk);
        self.registers[x as usize] = sum;
    }

    pub fn x8xy0(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    }

    pub fn x8xy4(&mut self, x: u8, y: u8) {
        self.registers[0xF] = 0;

        let (sum, overflowed) =
            self.registers[x as usize].overflowing_add(self.registers[y as usize]);

        self.registers[x as usize] = sum;

        if overflowed {
            self.registers[0xF] = 1;
        }
    }

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

    pub fn x8xy2(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vx & vy;
    }

    pub fn x8xy1(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vx | vy;
    }

    pub fn x8xy3(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        self.registers[x as usize] = vx ^ vy;
    }

    pub fn xannn(&mut self, nnn: u16) {
        self.index_register = nnn;
    }

    pub fn x8xy6(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];

        let shifted_vy = vy.wrapping_shr(1);
        self.registers[x as usize] = shifted_vy;
        self.registers[0xF] = 0x01 & vy;
    }

    pub fn x8xye(&mut self, x: u8, y: u8) {
        let vy = self.registers[y as usize];

        let shifted_vy = vy.wrapping_shl(1);
        self.registers[x as usize] = shifted_vy;
        self.registers[0xF] = 0x80 & vy;
    }

    pub fn xcxnn(&mut self) {
        todo!("impl rng");
    }

    pub fn xbnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.registers[0x0] as u16;
    }

    pub fn x3xnn(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.pc += 2;
        }
    }

    pub fn x5xy0(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        if vx == vy {
            self.pc += 2;
        }
    }

    pub fn x4xnn(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.pc += 2;
        }
    }

    pub fn x9xy0(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        if vx != vy {
            self.pc += 2;
        }
    }

    pub fn xfx15(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.delay_timer = vx;
    }

    pub fn xfx07(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    }

    pub fn xfx18(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.sound_timer = vx;
    }

    pub fn xfx0a(&mut self, x: u8) {
        todo!("take user input and store in vx");
    }

    pub fn xex9e(&mut self) {}

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

        let mut col = 0;
        for pixel in self.display {
            if col == VIDEO_WIDTH {
                println!();
                col = 0;
            }
            print!(
                "{}",
                match pixel {
                    0 => "0",
                    _ => "1",
                }
            );
            col += 1;
            std::io::stdout().flush().unwrap();
        }
        println!("\n");
    }
}
