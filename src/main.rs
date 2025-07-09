use std::{env::current_dir, io::Write, thread::current};

const HALF_BYTE: u32 = 4;
const FULL_BYTE: u32 = 8;

const VIDEO_WIDTH: usize = 64;
const VIDEO_HEIGHT: usize = 32;
const START_ADDRESS: u16 = 0x200;

pub fn main() {
    let mut mem: [u8; 4000] = [0_u8; 4000];

    let mut display: [u8; VIDEO_HEIGHT * VIDEO_WIDTH] = [0; VIDEO_HEIGHT * VIDEO_WIDTH];

    let mut pc: u16 = START_ADDRESS;

    let mut index_register: u16 = 0;

    let mut stack_pointer: u8 = 0;

    let mut stack: Vec<u16> = vec![];

    let mut delay_timer: u8 = 60;

    let mut sound_timer: u8 = 60;

    let mut registers: [u8; 16] = [0; 16];

    let program = std::fs::read("IBM Logo.ch8").unwrap();
    let mut memory_index = START_ADDRESS as usize;

    for byte in program {
        // mem[memory_index] = u8::from_str_radix(byte, 16).unwrap();
        mem[memory_index] = byte;
        memory_index += 1;
    }

    loop {
        // println!("pc: {}", pc);

        let opcode =
            u16::from(mem[pc as usize]).wrapping_shl(FULL_BYTE) | mem[pc as usize + 1] as u16;

        let instruction_prefix: u8 = (opcode & 0xF000).wrapping_shr(FULL_BYTE + HALF_BYTE) as u8;
        let nibble_0: u8 = (opcode & 0x0F00).wrapping_shr(FULL_BYTE) as u8;
        let nibble_1: u8 = (opcode & 0x00F0).wrapping_shr(HALF_BYTE) as u8;
        let nibble_2: u8 = (opcode & 0x000F) as u8;

        // let pc_0 = mem[pc as usize];
        // let pc_1 = mem[pc as usize + 1];
        // // 0b11110000 of pc
        // let instruction_prefix: u8 = (pc_0 & 0xF0).wrapping_shr(HALF_BYTE);
        // // 0b00001111 of pc
        // let nibble_0 = pc_0 & 0x0F;
        // // 0b11110000 of pc + 1
        // let nibble_1 = (pc_1 & 0xF0).wrapping_shr(HALF_BYTE);
        // // 0b00001111 of pc + 1
        // let nibble_2 = pc_1 & 0x0F;

        // 0b000011111111 of pc & pc + 1
        let nnn: u16 = u16::from(nibble_0).wrapping_shl(FULL_BYTE)
            | nibble_1.wrapping_shl(HALF_BYTE) as u16
            | nibble_2 as u16;
        let kk: u8 = nibble_1.wrapping_shl(HALF_BYTE) | nibble_2;

        pc += 2;

        match (instruction_prefix, nibble_0, nibble_1, nibble_2) {
            // clear screen
            (0, 0, 0xE, 0) => {
                // println!("clear screen");
                for pixel_idx in 0..display.len() {
                    display[pixel_idx] = 0;
                }
            }
            // 1nnn: JP addr
            (1, _, _, _) => {
                // println!("jump");
                pc = nnn;
            }
            // 6xkk: LD x, byte
            (6, _, _, _) => {
                // println!("set vx");
                registers[nibble_0 as usize] = kk;
            }

            // 8xy0: LD x, y
            (8, _, _, 0) => {
                // println!("load y into x");
                registers[nibble_0 as usize] = registers[nibble_1 as usize];
            }

            // 7xkk, ADD x, byte
            (7, _, _, _) => {
                // println!("set nn to vx");
                let (sum, _) = registers[nibble_0 as usize].overflowing_add(kk);
                registers[nibble_0 as usize] = sum;
            }
            // Annn: LD I, addr
            (0xA, _, _, _) => {
                // println!("set index register");
                index_register = nnn;
            }
            // Dxyn: DRW x, y, nibble
            (0xD, _, _, _) => {
                // println!("draw screen");
                let x_pos = registers[nibble_0 as usize] as usize % VIDEO_WIDTH;
                let y_pos = registers[nibble_1 as usize] as usize % VIDEO_HEIGHT;
                let height = nibble_2;

                registers[0xF] = 0;

                for row in 0..height as usize {
                    // one byte represents 8 columns of pixels, hence the shifting below
                    // ie, byte 7 represents 0000111 pixels
                    let sprite_byte = mem[index_register as usize + row as usize];
                    for col in 0..8 {
                        let sprite_bit = sprite_byte & (0x80_u8 >> col);

                        let current_display_state =
                            &mut display[(y_pos + row) * VIDEO_WIDTH + (x_pos + col)];

                        match (sprite_bit != 0, current_display_state.to_owned() != 0) {
                            (true, true) => {
                                *current_display_state ^= 0xFF;
                                registers[0xF] = 1;
                            }
                            (true, false) => *current_display_state ^= 0xFF,
                            (_, _) => continue,
                        }
                    }
                }

                let mut col = 0;
                for pixel in display {
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
            // _ => println!("IMPLEMENT {}{}{}{}", nib_0, nib_1, nib_2, nib_3),
            _ => {
                println!("no match");
                continue;
            }
        }
        if delay_timer > 0 {
            delay_timer -= 1;
        }

        if sound_timer > 0 {
            sound_timer -= 1;
        }
    }
}
