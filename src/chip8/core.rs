use std::fs;
use std::io;
use std::io::Read;

static FONT_SET: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0,
                             0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90,
                             0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0,
                             0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90,
                             0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90,
                             0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0,
                             0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80,
                             0xF0, 0x80, 0x80];


pub enum KeyEvent {
    Up(usize),
    Down(usize),
}

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
            pc: 0x200,
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
            c8.memory[i] = FONT_SET[i];
        }

        c8
    }

    /// Reads a byte from memory at the specified address `addr`.
    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    /// Writes `data` in memory at the specified address `addr`.
    pub fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    // Loads a game from a file at the specfied `path`.
    pub fn load_rom_file(&mut self, path: &str) -> io::Result<()> {
        let mut file = fs::File::open(path)?;
        let mut buffer : Vec<u8> = Vec::new();

        file.read_to_end(&mut buffer)?;

        for i in 0..buffer.len() { 
            self.memory[0x200 + i] = buffer[i];
        }

        Ok(())
    }

    pub fn step(&mut self) {
        if self.stopped {
            return;
        }

        self.opcode = combine_bytes(self.read(self.pc + 1), self.read(self.pc));

        // decode
        let func = opcodes::decode(self.opcode);

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

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        if self.stopped {
            self.stopped = false;
        }

        match key_event {
            KeyEvent::Up(key) => self.keypad[key] = 1,
            KeyEvent::Down(key) => self.keypad[key] = 0,
        }
    }

    pub fn get_framebuffer(&self) -> &[u8] { &self.vram }

    pub fn should_draw(&self) -> bool { self.draw_flag }
}

fn combine_bytes(low: u8, high: u8) -> u16 {
    (high as u16) << 8 | low as u16
}


mod opcodes {
    use rand;
    use rand::Rng;
    use chip8::core::Chip8;

    pub type OpcodeFunc = fn(&mut Chip8);

    pub fn decode(opcode: u16) -> OpcodeFunc {
        match opcode {
            0x00E0 => clear_screen,
            0x00EE => return_from_sub,
            0x1000...0x1FFF => jump_addr,
            0x2000...0x2FFF => call_sub_at_nnn,
            0x3000...0x3FFF => skip_if_vx_equal_to_nn,
            0x4000...0x4FFF => skip_if_vx_not_equal_to_nn,
            0x5000...0x5FFF => skip_if_vx_equal_to_vy,
            0x6000...0x6FFF => set_vx_to_immediate,
            0x7000...0x7FFF => add_nn_to_vx,
            0x8000...0x8FFF => {
                match opcode & 0xF {
                    0x0 => assign_vy_to_vx,
                    0x1 => vx_or_vy,
                    0x2 => vx_and_vy,
                    0x3 => vx_xor_vy,
                    0x4 => add_vy_to_vx,
                    0x5 => sub_vy_to_vx,
                    0x6 => shift_vx_right,
                    0x7 => sub_vx_to_vy,
                    0xE => shift_vx_left,
                    _ => panic!("Unknown opcode ${:04x}", opcode),
                }
            },
            0x9000...0x9FFF => skip_if_vx_not_equal_to_vy,
            0xA000...0xAFFF => set_memory_nnn,
            0xB000...0xBFFF => jump_addr_sum,
            0xC000...0xCFFF => rand_to_vx,
            0xD000...0xDFFF => draw,
            0xE000...0xEFFF => {
                match opcode & 0xFF {
                    0x9E => skip_if_key_pressed,
                    0xA1 => skip_if_key_not_pressed,
                    _ => panic!("Unknown opcode ${:04x}", opcode),
                }
            },
            0xF000...0xFFFF => {
                match opcode & 0xFF {
                    0x07 => set_vx_to_delay,
                    0x0A => wait_for_key_press,
                    0x15 => set_delay_to_vx,
                    0x18 => set_sound_to_vx,
                    0x1E => add_vx_to_i,
                    0x29 => set_i_to_sprite_addr,
                    0x33 => set_bcd,
                    0x55 => dump_registers,
                    0x65 => load_registers,
                    _ => panic!("Unknown opcode ${:04x}", opcode),
                }
            },
            _ => panic!("Unknown opcode ${:04x}", opcode),
        }
    }

    /// Extracts the X and Y parameters from a 16-bit opcode in the format 0x_XY_
    fn get_opcode_args(opcode: u16) -> (usize, usize) {
        ( ((opcode & 0x0F00) >> 8) as usize , ((opcode & 0x00F0) >> 4) as usize )
    }

    /// Extracts an 8-bit immediate value (NN)
    fn get_immediate_value(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    /// opcode 6XNN
    fn set_vx_to_immediate(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);

        c8.v[x] = nn;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 00E0.
    /// Resets the screen pixel values
    fn clear_screen(c8: &mut Chip8) {
        for i in 0..c8.vram.len() {
            c8.vram[i] = 0;
        }
        
        c8.draw_flag = true;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 00EE.
    /// Returns from a subroutine, meaning it will set the PC to the last stack value.
    fn return_from_sub(c8: &mut Chip8) {
        c8.sp -= 1;
        c8.pc = c8.stack[c8.sp as usize].wrapping_add(2);
    }

    /// opcode 1NNN.
    /// Sets the program counter to NNN.
    fn jump_addr(c8: &mut Chip8) {
        c8.pc = c8.opcode & 0x0FFF;
    }

    /// opcode 2NNN.
    /// It will call the subroutine at address NNN, i.e. move the PC to it.
    fn call_sub_at_nnn(c8: &mut Chip8) {
        c8.stack[c8.sp as usize] = c8.pc;
        c8.sp += 1;
        c8.pc = c8.opcode & 0x0FFF;
    }

    /// opcode 3XNN.
    /// It will skip the next instruction if Vx == NN.
    fn skip_if_vx_equal_to_nn(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);

        c8.pc = c8.pc.wrapping_add(if c8.v[x] == nn { 4 } else { 2 });
    }

    /// opcode 4XNN.
    /// It will skip the next instruction if Vx != NN.
    fn skip_if_vx_not_equal_to_nn(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);

        c8.pc = c8.pc.wrapping_add(if c8.v[x] != nn { 4 } else { 2 });
    }

    /// opcode 5XY0.
    /// It will skip the next instruction if Vx == Vy.
    fn skip_if_vx_equal_to_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.pc = c8.pc.wrapping_add(if c8.v[x] == c8.v[y] { 4 } else { 2 });
    }

    /// opcode 7XNN
    /// It will add NN to the Vx register
    fn add_nn_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);
        c8.v[x] = c8.v[x].wrapping_add(nn);
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY0
    /// Assigns the value of Vy to Vx
    fn assign_vy_to_vx(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.v[y];
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY1
    /// Assigns the value of Vx | Vy to Vx
    fn vx_or_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] |= c8.v[y];
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY2
    /// Assigns the value of Vx & Vy to Vx
    fn vx_and_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] &= c8.v[y];
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY3
    /// Assigns the value of Vx xor Vy to Vx
    fn vx_xor_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] ^= c8.v[y];
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY4
    /// Math	Vx += Vy	Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    fn add_vy_to_vx(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.v[x] = c8.v[x].wrapping_add(c8.v[y]);

        c8.v[0xF] = if let None = u8::checked_add(c8.v[x], c8.v[y])
                    { 1 } else { 0 };

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY5
    /// Math	Vx -= Vy	VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub_vy_to_vx(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.v[x] = c8.v[x].wrapping_sub(c8.v[y]);

        c8.v[0xF] = if let None = u8::checked_sub(c8.v[x], c8.v[y])
                    { 0 } else { 1 };

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY6
    /// BitOp	Vx >> 1	Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift.[2]
    fn shift_vx_right(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        c8.v[0xF] = c8.v[x] & 1;
        c8.v[x] >>= 1;

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XY7
    /// Math	Vx=Vy-Vx	Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub_vx_to_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.v[x] = c8.v[y].wrapping_sub(c8.v[x]);
        c8.v[0xF] = if let None = u8::checked_sub(c8.v[y], c8.v[x])
                    { 0 } else { 1 };

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 8XYE
    /// BitOp	Vx << 1	Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shift.[2]
    fn shift_vx_left(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        c8.v[0xF] = (c8.v[x] >> 7) & 0x1;
        c8.v[x] <<= 1;

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode 9XY0
    /// Cond	if(Vx!=Vy)	Skips the next instruction if VX doesn't equal VY.
    fn skip_if_vx_not_equal_to_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.pc = c8.pc.wrapping_add(if c8.v[x] != c8.v[y] { 4 } else { 2 });
    }

    /// opcode ANNN
    /// MEM	I = NNN	Sets I to the address NNN.
    fn set_memory_nnn(c8: &mut Chip8) {
        c8.i = c8.opcode & 0x0FFF;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode BNNN
    /// Flow PC=V0+NNN	Jumps to the address NNN plus V0.
    fn jump_addr_sum(c8: &mut Chip8) {
        c8.pc = (c8.opcode & 0x0FFF).wrapping_add(c8.v[0] as u16);
    }

    /// opcode CXNN
    /// Rand Vx=rand()&NN	Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    fn rand_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);
        c8.v[x] = rand::thread_rng().gen_range(0, 255) & nn;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode DXYN
    /// Disp	draw(Vx,Vy,N)	Draws a sprite at coordinate (VX, VY)
    fn draw(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        let height = (c8.opcode & 0xF) as usize;

        c8.v[0xF] = 0;

        for row in 0..height {
            let pixel_row = c8.memory[(c8.i as usize) + row];

            for col in 0..8 {
                // check if pixel went from 0 to 1
                let col_mask = 0x80 >> col;
                let pixel_updated = col_mask & pixel_row != 0;
                let pixel_address = x + col + ((y + row) * 64);

                if pixel_updated {
                    // if pixel was already 1, there's a collision
                    let collision = c8.vram[pixel_address] == 1;

                    if collision {
                        c8.v[0xF] = 1;
                    }

                    // flip the pixel
                    c8.vram[pixel_address] ^= 1;
                }
            }
        }

        c8.draw_flag = true;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode EX9E
    /// KeyOp	if(key()==Vx)	Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_if_key_pressed(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.pc = c8.pc.wrapping_add(if c8.keypad[c8.v[x] as usize] != 0 { 4 } else { 2 });
    }

    /// opcode EXA1
    /// KeyOp	if(key()!=Vx)	Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_if_key_not_pressed(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.pc = c8.pc.wrapping_add(if c8.keypad[c8.v[x] as usize] == 0 { 4 } else { 2 });
    }

    /// opcode FX07
    /// Timer	Vx = get_delay()	Sets VX to the value of the delay timer.
    fn set_vx_to_delay(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.delay_t;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX0A
    /// KeyOp	Vx = get_key()	A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
    fn wait_for_key_press(c8: &mut Chip8) {
        c8.stopped = true;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX15
    /// Timer	delay_timer(Vx)	Sets the delay timer to VX.
    fn set_delay_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.delay_t = c8.v[x];
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX18
    /// Sound	sound_timer(Vx)	Sets the sound timer to VX.
    fn set_sound_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.sound_t = c8.v[x];
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX1E
    /// MEM	I +=Vx	Adds VX to I.[3]
    fn add_vx_to_i(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        c8.v[0xF] = if (c8.i + c8.v[x] as u16) > 0x0FFF { 1 } else { 0 };
        c8.i += c8.v[x] as u16;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX29
    /// MEM	I=sprite_addr[Vx]	Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    fn set_i_to_sprite_addr(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.i = (c8.v[x] as u16) * 5;
        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX33
    /// BCD	set_BCD(Vx);
    fn set_bcd(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let bcd_value = c8.v[x];
        let addr = c8.i;

        c8.write(addr, bcd_value / 100);
        c8.write(addr + 1, (bcd_value % 100) / 10);
        c8.write(addr + 2, (bcd_value % 100) % 10);

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX55
    /// MEM	reg_dump(Vx,&I)	Stores V0 to VX (including VX) in memory starting at address I.[4]
    fn dump_registers(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        for i in 0..x {
            let data = c8.v[i];
            let addr = c8.i + i as u16;
            c8.write(addr, data);
        }

        c8.pc = c8.pc.wrapping_add(2);
    }

    /// opcode FX65
    /// MEM	reg_load(Vx,&I)	Fills V0 to VX (including VX) with values from memory starting at address I.[4]
    fn load_registers(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        for i in 0..x {
            c8.v[i] = c8.read(c8.i + i as u16);
        }

        c8.pc = c8.pc.wrapping_add(2);
    }
}