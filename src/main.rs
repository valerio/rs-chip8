extern crate sdl2;
extern crate rand;
extern crate clap;

mod chip8;

use clap::{Arg, App};

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

    chip8::io::run_emulator(&file_name).expect("Error occurred in main loop");
}