extern crate sdl2;

use std::{sync::Arc, time::Duration};

use sdl2::{
    EventPump, Sdl,
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{FRect, Point, Rect},
    render::Canvas,
    video::Window,
};

use crate::chip8::{Chip8, VIDEO_HEIGHT, VIDEO_WIDTH};
pub mod chip8;

// const ROM: &str = "Tetris [Fran Dachille, 1991].ch8";
const ROM: &str = "IBM Logo.ch8";
// const ROM: &str = "test_opcode.ch8";
// const ROM: &str = "Tank.ch8";

const PIXEL_SIZE: u32 = 8;

pub fn main() {
    let mut emulator = Chip8::new();
    emulator.load_program(std::fs::read(ROM).unwrap());
    emulator.fetch_decode_execute();
}
pub struct Renderer {
    context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl Renderer {
    pub fn init() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "Chip-8 Emulator",
                PIXEL_SIZE * VIDEO_WIDTH as u32,
                PIXEL_SIZE * VIDEO_HEIGHT as u32,
            )
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump().unwrap();

        canvas
            .set_logical_size(
                PIXEL_SIZE * VIDEO_WIDTH as u32,
                PIXEL_SIZE * VIDEO_HEIGHT as u32,
            )
            .unwrap();

        Self {
            context: sdl_context,
            canvas,
            event_pump,
        }
    }

    pub fn draw(&mut self, display: &[u8; VIDEO_HEIGHT * VIDEO_WIDTH], keypad: &mut [u8; 16]) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyUp {
                    keycode: Some(key), ..
                }
                | Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::A
                    | Keycode::B
                    | Keycode::C
                    | Keycode::D
                    | Keycode::E
                    | Keycode::F
                    | Keycode::Num0
                    | Keycode::Num1
                    | Keycode::Num2
                    | Keycode::Num3
                    | Keycode::Num4
                    | Keycode::Num5
                    | Keycode::Num6
                    | Keycode::Num7
                    | Keycode::Num8
                    | Keycode::Num9 => {
                        let hex_key = u8::from_str_radix(
                            &char::from_u32(key.into_i32() as u32).unwrap().to_string(),
                            16,
                        )
                        .unwrap();
                        println!("received {}", hex_key);

                        match event {
                            Event::KeyUp { .. } => keypad[hex_key as usize] = 0,
                            Event::KeyDown { .. } => keypad[hex_key as usize] = 1,
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // game here

        // let origin_x = WINDOW_WIDTH / VIDEO_WIDTH as u32 / 2;
        // let origin_y = WINDOW_HEIGHT / VIDEO_HEIGHT as u32 / 2;

        let mut col = 0;
        let mut row = 0;

        for pixel in display {
            if col == VIDEO_WIDTH as usize {
                col = 0;
                row += 1;
            }
            // if row == VIDEO_HEIGHT as usize {
            //     // row = 0;
            // }

            self.canvas.set_draw_color(Color::BLUE);
            if pixel != &0 {
                self.canvas.set_draw_color(Color::WHITE);
                self.canvas
                    .fill_rect(Rect::new(
                        PIXEL_SIZE as i32 * col as i32,
                        PIXEL_SIZE as i32 * row as i32,
                        PIXEL_SIZE,
                        PIXEL_SIZE,
                    ))
                    .unwrap();
            } else {
                self.canvas
                    .fill_rect(Rect::new(
                        PIXEL_SIZE as i32 * col as i32,
                        PIXEL_SIZE as i32 * row as i32,
                        PIXEL_SIZE,
                        PIXEL_SIZE,
                    ))
                    .unwrap();
            }
            col += 1;
        }

        self.canvas.present();
        // std::thread::sleep(Duration::new(0, 1_000_000u32 / 60));
    }
}
