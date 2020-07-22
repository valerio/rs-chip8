mod chip8;

use chip8::Chip8;
use clap::{App, Arg};
use ggez::*;

struct State {
    emulator: Chip8,
    canvas: graphics::Canvas,
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // TODO: move this to the emulator (run until next frame)
        while !self.emulator.should_draw() {
            self.emulator.step();
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.emulator.should_draw() {
            return Ok(());
        }

        graphics::clear(ctx, (0, 0, 0, 1).into());
        graphics::set_canvas(ctx, Some(&self.canvas));

        // TODO: draw the framebuffer

        Ok(())
    }
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

    let c = conf::Conf::new();
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("rs-chip8", "valerio")
        .conf(c)
        .build()
        .expect("Could not build ggez context");

    let state = &mut State {
        emulator,
        canvas: graphics::Canvas::with_window_size(ctx).unwrap(),
    };

    event::run(ctx, event_loop, state).unwrap();
}
