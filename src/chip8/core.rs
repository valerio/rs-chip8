
static FONT_SET: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0,
                             0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90,
                             0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0,
                             0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90,
                             0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90,
                             0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0,
                             0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80,
                             0xF0, 0x80, 0x80];


type OpcodeFunc = fn(&mut Chip8);

#[derive(Debug)]
pub struct Chip8 {
    i: u16,
    pc: u16,
    sp: u16,
    stack: Vec<u16>,
    v: Vec<u8>,
    memory: Vec<u8>,
    vram: Vec<u8>,
    keypad: Vec<u8>,
    delay_t: u8,
    sound_t: u8,
    opcode: u16,
    draw_flag: bool,
    stopped: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c8 = Chip8 {
            i: 0,
            pc: 0,
            sp: 0,
            stack: vec![0; 16],
            v: vec![0; 16],
            memory: vec![0; 4096],
            vram: vec![0; 64 * 32],
            keypad: vec![0; 16],
            delay_t: 0,
            sound_t: 0,
            opcode: 0,
            draw_flag: false,
            stopped: false,
        };

        for i in 0..FONT_SET.len() {
            c8.v[i] = FONT_SET[i];
        }

        c8
    }

    pub fn step(&mut self) {
        let opcode = combine_bytes(self.memory[(self.pc + 1) as usize],
                                   self.memory[self.pc as usize]);

        // decode
        let func = decode(opcode);

        // exec
        func(self);

        // update timers
        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            if self.sound_t == 1 {
                println!("BOOP!");
            }
            self.sound_t -= 1;
        }
    }
}

fn combine_bytes(low: u8, high: u8) -> u16 {
    (high as u16) << 8 | low as u16
}

fn decode(opcode: u16) -> OpcodeFunc {
    // TODO: map all opcodes
    match opcode {
        _ => nop,
    }
}

fn nop(c8: &mut Chip8) {
    c8.pc += 2;
}