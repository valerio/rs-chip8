
static FONT_SET: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0,
                             0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90,
                             0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0,
                             0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90,
                             0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90,
                             0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0,
                             0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80,
                             0xF0, 0x80, 0x80];


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

    /// Reads a byte from memory at the specified address `addr`.
    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    /// Writes `data` in memory at the specified address `addr`.
    pub fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn step(&mut self) {
        let opcode = combine_bytes(self.read(self.pc + 1), self.read(self.pc));

        // decode
        let func = opcodes::decode(opcode);

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


mod opcodes {
    use chip8::core::Chip8;

    pub type OpcodeFunc = fn(&mut Chip8);

    pub fn decode(opcode: u16) -> OpcodeFunc {
        // TODO: map all opcodes
        match opcode {
            0x6000...0x6FFF => set_vx_to_immediate,
            _ => nop,
        }
    }

    fn nop(c8: &mut Chip8) {
        c8.pc += 2;
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
        c8.pc += 2;
    }

    /// opcode 00E0.
    /// Resets the screen pixel values
    fn clear_screen(c8: &mut Chip8) {
        for i in 0..c8.vram.len() {
            c8.vram[i] = 0;
        }

        c8.pc += 2;
    }

    /// opcode 00EE.
    /// Returns from a subroutine, meaning it will set the PC to the last stack value.
    fn return_from_sub(c8: &mut Chip8) {
        c8.sp -= 1;
        c8.pc = c8.stack[c8.sp as usize] + 2;
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

        c8.pc += if c8.v[x] == nn { 4 } else { 2 };
    }

    /// opcode 4XNN.
    /// It will skip the next instruction if Vx != NN.
    fn skip_if_vx_not_equal_to_nn(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);

        c8.pc += if c8.v[x] == nn { 2 } else { 4 };
    }

    /// opcode 5XY0.
    /// It will skip the next instruction if Vx == Vy.
    fn skip_if_vx_equal_to_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.pc += if c8.v[x] == c8.v[x] { 4 } else { 2 };
    }

    /// opcode 7XNN
    /// It will add NN to the Vx register
    fn add_nn_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);
        c8.v[x] += nn;
        c8.pc += 2;
    }

    /// opcode 8XY0
    /// Assigns the value of Vy to Vx
    fn assign_vy_to_vx(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.v[y];
        c8.pc += 2;
    }

    /// opcode 8XY1
    /// Assigns the value of Vx | Vy to Vx
    fn vx_or_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.v[x] | c8.v[y];
        c8.pc += 2;
    }

    /// opcode 8XY2
    /// Assigns the value of Vx & Vy to Vx
    fn vx_and_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.v[x] & c8.v[y];
        c8.pc += 2;
    }

    /// opcode 8XY3
    /// Assigns the value of Vx xor Vy to Vx
    fn vx_xor_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.v[x] ^ c8.v[y];
        c8.pc += 2;
    }

    /// opcode 8XY4
    /// Math	Vx += Vy	Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    fn add_vy_to_vx(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.v[x] = c8.v[x] + c8.v[y];

        c8.v[0xF] = if let None = u8::checked_add(c8.v[x], c8.v[y])
                    { 1 } else { 0 };

        c8.pc += 2;
    }

    /// opcode 8XY5
    /// Math	Vx -= Vy	VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub_vy_to_vx(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.v[x] = c8.v[x] - c8.v[y];

        c8.v[0xF] = if let None = u8::checked_sub(c8.v[x], c8.v[y])
                    { 1 } else { 0 };

        c8.pc += 2;
    }

    /// opcode 8XY6
    /// BitOp	Vx >> 1	Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift.[2]
    fn shift_vx_right(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        let lsb = x & 1;
        c8.v[x] = c8.v[x] >> 1;
        c8.v[0xF] = lsb as u8;

        c8.pc += 2;
    }

    /// opcode 8XY7
    /// Math	Vx=Vy-Vx	Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub_vx_to_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.v[x] = c8.v[y] - c8.v[x];

        c8.v[0xF] = if let None = u8::checked_sub(c8.v[y], c8.v[x])
                    { 1 } else { 0 };

        c8.pc += 2;
    }

    /// opcode 8XYE
    /// BitOp	Vx << 1	Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shift.[2]
    fn shift_vx_left(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        let msb = x & 0x80;
        c8.v[x] = c8.v[x] << 1;
        c8.v[0xF] = msb as u8;

        c8.pc += 2;
    }

    /// opcode 9XY0
    /// Cond	if(Vx!=Vy)	Skips the next instruction if VX doesn't equal VY.
    fn skip_if_vx_not_equal_to_vy(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);

        c8.pc += if c8.v[x] == c8.v[x] { 2 } else { 4 };
    }

    /// opcode ANNN
    /// MEM	I = NNN	Sets I to the address NNN.
    fn set_memory_nnn(c8: &mut Chip8) {
        c8.i = c8.opcode & 0x0FFF;
        c8.pc += 2;
    }

    /// opcode BNNN
    /// Flow PC=V0+NNN	Jumps to the address NNN plus V0.
    fn jump_addr_sum(c8: &mut Chip8) {
        c8.pc = (c8.opcode & 0x0FFF) + (c8.v[0] as u16);
    }

    /// opcode CXNN
    /// Rand Vx=rand()&NN	Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    fn rand_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let nn = get_immediate_value(c8.opcode);
        // TODO: put a random number between 0 and 255 here
        c8.v[x] = 0 & nn;
        c8.pc += 2;
    }

    /// opcode DXYN
    /// Disp	draw(Vx,Vy,N)	Draws a sprite at coordinate (VX, VY)
    fn draw(c8: &mut Chip8) {
        let (x, y) = get_opcode_args(c8.opcode);
        let height = c8.opcode & 0xF;

        c8.v[0xF] = 0;

        for row in 0..height {

            let pixelRow = c8.memory[(c8.i as usize) + row as usize];

            for col in 0..8 {
                // check if pixel went from 0 to 1
                let colMask = 0x80 >> col;
                let pixelUpdated = (colMask & pixelRow) != 0;
                let pixelAddress = x + col + ((y + row as usize) * 64);

                if pixelUpdated {
                    // if pixel was already 1, there's a collision
                    let collision = c8.vram[pixelAddress] == 1;

                    if collision {
                        c8.v[0xF] = 1;
                    }

                    // flip the pixel
                    c8.vram[pixelAddress] ^= 1;
                }
            }
        }

        c8.draw_flag = true;
        c8.pc += 2;
    }

    /// opcode EX9E
    /// KeyOp	if(key()==Vx)	Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_if_key_pressed(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.pc += if c8.keypad[x] != 0 { 4 } else { 2 };
    }

    /// opcode EXA1
    /// KeyOp	if(key()!=Vx)	Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_if_key_not_pressed(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.pc += if c8.keypad[x] != 0 { 2 } else { 4 };
    }

    /// opcode FX07
    /// Timer	Vx = get_delay()	Sets VX to the value of the delay timer.
    fn set_vx_to_delay(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.v[x] = c8.delay_t;
        c8.pc += 2;
    }

    /// opcode FX0A
    /// KeyOp	Vx = get_key()	A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
    fn wait_for_key_press(c8: &mut Chip8) {
        c8.stopped = true;
        c8.pc += 2;
    }

    /// opcode FX15
    /// Timer	delay_timer(Vx)	Sets the delay timer to VX.
    fn set_delay_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.delay_t = c8.v[x];
        c8.pc += 2;
    }

    /// opcode FX18
    /// Sound	sound_timer(Vx)	Sets the sound timer to VX.
    fn set_sound_to_vx(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.sound_t = c8.v[x];
        c8.pc += 2;
    }

    /// opcode FX1E
    /// MEM	I +=Vx	Adds VX to I.[3]
    fn add_vx_to_i(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.i += c8.v[x] as u16;
        c8.pc += 2;
    }

    /// opcode FX29
    /// MEM	I=sprite_addr[Vx]	Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    fn set_i_to_sprite_addr(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        c8.i = (c8.v[x] * 5) as u16;
        c8.pc += 2;
    }

    /// opcode FX33
    /// BCD	set_BCD(Vx);
    fn set_bcd(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);
        let bcdValue = c8.v[x];

        let addr = c8.i;
        c8.write(addr, bcdValue / 100);
        c8.write(addr + 1, (bcdValue % 100) / 10);
        c8.write(addr + 2, (bcdValue % 100) % 10);

        c8.pc += 2;
    }

    /// opcode FX55
    /// MEM	reg_dump(Vx,&I)	Stores V0 to VX (including VX) in memory starting at address I.[4]
    fn dump_registers(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        for i in 0..x {
            let data = c8.v[i];
            let addr = (c8.i as u16) + i as u16;
            c8.write(addr, data);
        }

        c8.pc += 2;
    }

    /// opcode FX65
    /// MEM	reg_load(Vx,&I)	Fills V0 to VX (including VX) with values from memory starting at address I.[4]
    fn load_registers(c8: &mut Chip8) {
        let (x, _) = get_opcode_args(c8.opcode);

        for i in 0..x {
            c8.v[i] = c8.read((c8.i as u16) + i as u16);
        }

        c8.pc += 2;
    }
}