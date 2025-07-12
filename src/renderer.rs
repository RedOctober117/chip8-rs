use std::time::Duration;

use sdl2::{
    EventPump, Sdl,
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::Canvas,
    ttf::{self, Sdl2TtfContext},
    video::Window,
};

use crate::chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub struct Renderer {
    emulator: Chip8,
    _context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    ttf: Sdl2TtfContext,
    pixel_size: u32,
}

impl Renderer {
    pub fn init(emu: Chip8, window_width: u32, window_height: u32, pixel_size: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Chip-8 Emulator", window_width, window_height)
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
                pixel_size * DISPLAY_WIDTH as u32,
                pixel_size * DISPLAY_HEIGHT as u32,
            )
            .unwrap();

        let ttf = ttf::init().unwrap();

        Self {
            emulator: emu,
            _context: sdl_context,
            canvas,
            event_pump,
            ttf,
            pixel_size,
        }
    }

    pub fn cycle(&mut self) {
        let font = self.ttf.load_font("Px437_IBM_VGA_8x16.ttf", 16).unwrap();
        let texture_creator = self.canvas.texture_creator();

        let mut debug_mode = false;
        let mut fps_offset: f64 = 0_f64;

        'render_loop: loop {
            if debug_mode {
                self.emulator.debug_out();
            }

            let timer = self._context.timer().unwrap();
            let start = timer.performance_counter();
            self.emulator.cycle();

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::ESCAPE),
                        ..
                    } => {
                        break 'render_loop;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::EQUALS),
                        ..
                    } => {
                        fps_offset -= 1_f64;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::MINUS),
                        ..
                    } => {
                        fps_offset += 1_f64;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::BACKSPACE),
                        ..
                    } => debug_mode ^= true,
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
                            // println!("received {}", hex_key);

                            match event {
                                Event::KeyUp { .. } => self.emulator.key_up(hex_key),
                                Event::KeyDown { .. } => self.emulator.key_down(hex_key),
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            let mut col = 0;
            let mut row = 0;

            for pixel in self.emulator.get_display() {
                if col == DISPLAY_WIDTH as usize {
                    col = 0;
                    row += 1;
                }

                self.canvas.set_draw_color(Color::BLACK);
                if pixel != &0 {
                    self.canvas.set_draw_color(Color::GREEN);
                    self.canvas
                        .fill_rect(Rect::new(
                            self.pixel_size as i32 * col as i32,
                            self.pixel_size as i32 * row as i32,
                            self.pixel_size,
                            self.pixel_size,
                        ))
                        .unwrap();
                } else {
                    self.canvas
                        .fill_rect(Rect::new(
                            self.pixel_size as i32 * col as i32,
                            self.pixel_size as i32 * row as i32,
                            self.pixel_size,
                            self.pixel_size,
                        ))
                        .unwrap();
                }
                col += 1;
            }

            let mut end = timer.performance_counter();
            let mut elapsed: f64 = (end - start) as f64 / timer.performance_frequency() as f64;

            std::thread::sleep(Duration::from_millis(
                ((16.666_f64 + fps_offset) - (elapsed * 1000_f64).floor()) as u64,
            ));

            end = timer.performance_counter();
            elapsed = (end - start) as f64 / timer.performance_frequency() as f64;

            let text = font
                .render(&format!("FPS: {}", (1.0_f64 / elapsed) as u32))
                .solid(Color::WHITE)
                .unwrap();

            let destination = Rect::new(0, 0, text.width(), text.height());
            let text_texture = texture_creator.create_texture_from_surface(text).unwrap();

            self.canvas.copy(&text_texture, None, destination).unwrap();
            // println!("FPS: {}", 1.0_f64 / elapsed);

            self.canvas.present();
        }
    }
}
