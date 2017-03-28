extern crate sdl2;
extern crate rand;
extern crate clap;

mod chip8;

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use clap::{Arg, App};

use chip8::core::Chip8;

fn main() {
    let matches = App::new("rs-chip8")
                    .about("Chip8 Emulator")
                    .arg(Arg::with_name("file")
                               .value_name("FILE")
                               .help("Sets the input file to use")
                               .required(true)
                               .index(1))
                    .get_matches();

    let file_name = matches.value_of("file").expect("Must specify a file to load.");

    run_emulator(&file_name).expect("Error occurred in main loop");
}

fn run_emulator(file_name: &str) -> Result<(), Box<std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Chip8", 800, 600)
        .position_centered()
        .resizable()
        .opengl()
        .build()?;

    let mut renderer = window.renderer()
        .build()?;

    let mut texture = renderer
        .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)?;

    // Placeholder texture: make a black/white checkerboard
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..32 {
            for x in 0..64 {
                let offset = x * 3 + pitch * y;
                let color = if (x + y) % 2 == 0 { 255 } else { 0 };
                buffer[offset] = color;
                buffer[offset + 1] = color;
                buffer[offset + 2] = color;
            }
        }
    })?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut emulator = Chip8::new();
    emulator.load_rom_file(file_name)?;

    'running: loop {
        // Execute
        emulator.step();

        // Draw
        renderer.clear();
        renderer.copy(&texture, None, None)?;
        renderer.present();

        // Handle inputs
        for event in event_pump.poll_iter() {
            let key_event : Option<chip8::core::KeyEvent> = match event {
                Event::Quit {..} 
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    println!("Key {} down", key);
                    handle_input(true, key)
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    println!("Key {} up", key);
                    handle_input(false, key)
                }
                _ => {None}
            };

            if let Some(event) = key_event {
                emulator.handle_input(event);
            }
        }
    }

    Ok(())
}

fn handle_input(down: bool, key: sdl2::keyboard::Keycode) -> Option<chip8::core::KeyEvent> {
    use sdl2::keyboard::Keycode;
    use chip8::core::KeyEvent;

    let keypad_num : usize = match key {
        Keycode::Num1 => 1,
        Keycode::Num2 => 2,
        Keycode::Num3 => 3,
        Keycode::Num4 => 3,
        Keycode::Q => 5,
        Keycode::W => 6,
        Keycode::E => 7,
        Keycode::R => 8,
        Keycode::A => 9,
        Keycode::S => 0xA,
        Keycode::D => 0xB,
        Keycode::F => 0xC,
        Keycode::Z => 0xD,
        Keycode::X => 0xE,
        Keycode::C => 0,
        Keycode::V => 0xF,
        _ => return None,
    };

    if down {
        Some(KeyEvent::Down(keypad_num))
    } else {
        Some(KeyEvent::Up(keypad_num))
    }
}