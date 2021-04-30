mod chip8;

use chip8::Chip8;
use clap::{App, Arg};
use minifb::{Key, Window, WindowOptions};

use crate::chip8::KeyEvent;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn keys_to_event_vec(keys: Option<Vec<Key>>) -> Vec<usize> {
    keys.map_or(vec![], |keys| {
        keys.iter()
            .filter_map(|key| match key {
                Key::Key1 => Some(0),
                Key::Key2 => Some(1),
                Key::Key3 => Some(2),
                Key::Key4 => Some(3),
                Key::Q => Some(4),
                Key::W => Some(5),
                Key::E => Some(6),
                Key::R => Some(7),
                Key::A => Some(8),
                Key::S => Some(9),
                Key::D => Some(10),
                Key::F => Some(11),
                Key::Z => Some(12),
                Key::X => Some(13),
                Key::C => Some(14),
                Key::V => Some(15),
                _ => None,
            })
            .collect()
    })
}

fn main() {
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

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Chip8",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X8,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..Default::default()
        },
    )
    .expect("Could not initialize window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // handle keys pressed/released
        for key in keys_to_event_vec(window.get_keys_pressed(minifb::KeyRepeat::No)) {
            emulator.handle_input(KeyEvent::Down(key));
        }
        for key in keys_to_event_vec(window.get_keys_released()) {
            emulator.handle_input(KeyEvent::Up(key));
        }

        // execute until the next frame should be drawn
        while !emulator.should_draw() {
            emulator.step();
        }
        emulator.draw_flag = false;

        // prepare the framebuffer
        let raw_fb = emulator.get_framebuffer();
        for (i, pixel) in buffer.iter_mut().enumerate() {
            if i == raw_fb.len() {
                break;
            }

            *pixel = if raw_fb[i] == 0 {
                0xFF000000
            } else {
                0xFFFFFFFF
            };
        }

        // draw
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        // TODO: sound
    }
}
