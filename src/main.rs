mod chip8;

use chip8::Chip8;
use clap::{App, Arg};
use ggez::{
    conf::WindowSetup,
    event::{self, EventHandler, KeyCode},
    graphics::{self, Color, DrawParam, Image},
    input, timer, Context, ContextBuilder, GameResult,
};

use crate::chip8::KeyEvent;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const KEYS: [KeyCode; 16] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::R,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::C,
    KeyCode::V,
];

fn keycode_to_event(key: KeyCode) -> usize {
    match key {
        KeyCode::Key1 => 0,
        KeyCode::Key2 => 1,
        KeyCode::Key3 => 2,
        KeyCode::Key4 => 3,
        KeyCode::Q => 4,
        KeyCode::W => 5,
        KeyCode::E => 6,
        KeyCode::R => 7,
        KeyCode::A => 8,
        KeyCode::S => 9,
        KeyCode::D => 10,
        KeyCode::F => 11,
        KeyCode::Z => 12,
        KeyCode::X => 13,
        KeyCode::C => 14,
        KeyCode::V => 15,
        _ => 0,
    }
}

struct EmulatorState {
    emulator: Chip8,
    fb: Vec<u8>,
}

impl EmulatorState {
    pub fn new(emulator: Chip8) -> Self {
        let fb = vec![0; WIDTH * HEIGHT * 4];
        Self { emulator, fb }
    }
}

impl EventHandler for EmulatorState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        for key in KEYS {
            let input = if input::keyboard::is_key_pressed(ctx, key) {
                KeyEvent::Up(keycode_to_event(key))
            } else {
                KeyEvent::Down(keycode_to_event(key))
            };

            self.emulator.handle_input(input);
        }

        while timer::check_update_time(ctx, 120) {
            self.emulator.step();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if !self.emulator.should_draw() {
            return Ok(());
        }

        graphics::clear(ctx, Color::WHITE);

        let bw_framebuffer = self.emulator.get_framebuffer();
        for i in 0..bw_framebuffer.len() {
            let color = if bw_framebuffer[i] == 0 { 0 } else { 255 };
            self.fb[(i * 4)] = color;
            self.fb[(i * 4) + 1] = color;
            self.fb[(i * 4) + 2] = color;
            self.fb[(i * 4) + 3] = 255; // alpha
        }

        debug_assert_eq!(self.fb.len(), WIDTH * HEIGHT * 4);

        let img = Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, &self.fb)?;
        graphics::draw(ctx, &img, DrawParam::default())?;

        graphics::present(ctx)
    }
}

fn main() -> GameResult {
    let matches = App::new("rs-chip8")
        .about("Chip8 Emulator")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let file_name = matches
        .value_of("file")
        .expect("Must specify a file to load.");

    let mut emulator = Chip8::new();
    emulator
        .load_rom_file(file_name)
        .expect("Could not load file");

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("rs-chip8", "Valerio")
        .window_setup(WindowSetup {
            title: "Chip8".to_owned(),
            ..WindowSetup::default()
        })
        .build()?;

    let state = EmulatorState::new(emulator);

    graphics::set_resizable(&mut ctx, true)?;

    // Set screen coordinates to match the size of a Chip8 framebuffer.
    // This ensures that the framebuffer is stretched across the entire window.
    graphics::set_screen_coordinates(
        &mut ctx,
        graphics::Rect::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32),
    )?;

    // Set default filter mode to nearest-neighbor.
    // This makes low resolution images look crisp as they are upscaled, if left to default
    // Chip8 graphics would be smoothened out and look "blurry".
    graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);

    event::run(ctx, event_loop, state);
}
