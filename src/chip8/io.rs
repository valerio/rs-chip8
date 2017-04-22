use sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std;
use std::thread;
use std::time::Duration;

use chip8::core::{Chip8, KeyEvent};

pub fn run_emulator(file_name: &str) -> Result<(), Box<std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Chip8", 640, 320)
        .position_centered()
        .resizable()
        .opengl()
        .build()?;

    let mut renderer = window.renderer()
        .build()?;

    let mut texture = renderer
        .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut emulator = Chip8::new();
    emulator.load_rom_file(file_name)?;

    'running: loop {
        // Handle inputs
        for event in event_pump.poll_iter() {
            let key_event : Option<KeyEvent> = match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    // println!("Key {} down", key);
                    map_keycode(true, key)
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    // println!("Key {} up", key);
                    map_keycode(false, key)
                }
                _ => {None}
            };

            if let Some(event) = key_event {
                emulator.handle_input(event);
            }
        }

        // Execute
        emulator.step();

        // Draw
        draw_step(&mut renderer, &mut texture, &emulator)?;

        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

fn draw_step(renderer: &mut sdl2::render::Renderer, texture: &mut sdl2::render::Texture, emulator: &Chip8) -> 
        Result<(), Box<std::error::Error>> {
    renderer.clear();

    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        if !emulator.should_draw() { return; }

        let fb = emulator.get_framebuffer();

        for y in 0..32 {
            for x in 0..64 {
                let offset = x * 3 + pitch * y;
                let fb_index = x + (y * 64);
                let pixel = fb[fb_index];

                let color = if pixel == 0 { 0 } else { 255 };

                buffer[offset] = color;
                buffer[offset + 1] = color;
                buffer[offset + 2] = color;
            }
        }
    })?;

    renderer.copy(&texture, None, None)?;
    renderer.present();

    Ok(())
}

fn map_keycode(down: bool, key: sdl2::keyboard::Keycode) -> Option<KeyEvent> {
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