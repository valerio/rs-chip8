extern crate sdl2;

mod chip8;

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use chip8::core::Chip8;

fn main() {
    println!("Hello, world!");
    
    run_emulator().expect("Error occurred in main loop");
}

fn run_emulator() -> Result<(), Box<std::error::Error>> {
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

    'running: loop {
        // TODO: Run the chip8 here!
        emulator.step();

        // Drawing
        renderer.clear();
        renderer.copy(&texture, None, None)?;
        renderer.present();

        // Input handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} 
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    println!("Key {} down", key);
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    println!("Key {} up", key);
                }
                _ => {}
            }
        }
    }

    Ok(())
}