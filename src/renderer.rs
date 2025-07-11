use sdl2::{
    EventPump, Sdl, event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas,
    video::Window,
};

pub struct Renderer {
    _context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    pixel_size: u32,
    video_width: u32,
    video_height: u32,
}

impl Renderer {
    pub fn init(pixel_size: u32, video_width: u32, video_height: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "Chip-8 Emulator",
                pixel_size * video_width as u32,
                pixel_size * video_height as u32,
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
                pixel_size * video_width as u32,
                pixel_size * video_height as u32,
            )
            .unwrap();

        Self {
            _context: sdl_context,
            canvas,
            event_pump,
            pixel_size,
            video_width,
            video_height,
        }
    }

    pub fn draw(&mut self, display: &[u8], keypad: &mut [u8; 17]) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => {
                    keypad[16] = 1;
                }
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

        // let origin_x = WINDOW_WIDTH / self.video_width as u32 / 2;
        // let origin_y = WINDOW_HEIGHT / self.video_height as u32 / 2;

        let mut col = 0;
        let mut row = 0;

        for pixel in display {
            if col == self.video_width as usize {
                col = 0;
                row += 1;
            }
            // if row == self.video_height as usize {
            //     // row = 0;
            // }

            self.canvas.set_draw_color(Color::BLUE);
            if pixel != &0 {
                self.canvas.set_draw_color(Color::WHITE);
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

        self.canvas.present();
        // std::thread::sleep(Duration::new(0, 1_000_000u32 / 60));
    }
}
