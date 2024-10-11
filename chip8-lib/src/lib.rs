use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const NUM_REGS: usize = 16;
const MEMORY_SIZE: usize = 4096;

const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const ENTRY_POINT_ADDR: u16 = 0x200;

const SPRITE_WIDTH: u16 = 8;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Processor {
    pc: u16,
    mem: [u8; MEMORY_SIZE],
    screen: [bool; SCREEN_WIDTH*SCREEN_HEIGHT],
    registers: [u8; NUM_REGS],

    ip: u16,

    sp: u16,
    stack: [u16; STACK_SIZE],

    // Key Press
    keys: [bool; NUM_KEYS],

    delay_timer: u8,
    sound_timer: u8,
}

impl Processor {
    pub fn new() -> Self {
        let mut processor = Self {
            pc: ENTRY_POINT_ADDR,
            mem: [0; MEMORY_SIZE],
            screen: [false; SCREEN_HEIGHT*SCREEN_WIDTH],
            registers: [0; NUM_REGS],
            ip: 0,

            stack: [0; STACK_SIZE],
            sp: 0,

            keys: [false; NUM_KEYS],

            delay_timer: 0,
            sound_timer: 0,
        };

        processor.mem[0..FONTSET_SIZE].copy_from_slice(&FONTSET);
        processor
    }

    pub fn reset(&mut self) {
        self.pc = ENTRY_POINT_ADDR;
        self.mem = [0; MEMORY_SIZE];
        self.screen = [false; SCREEN_HEIGHT*SCREEN_WIDTH];
        self.registers = [0; NUM_REGS];
        self.ip = 0;
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.mem[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize]=val;
        self.sp+=1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize] as u16
    }

    pub fn tick(&mut self) {
        // Fetch Code
        let op = self.fetch();

        // Decode ?

        // Execute
        self.execute(op);

    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.mem[self.pc as usize] as u16;
        let lower_byte = self.mem[(self.pc + 1) as usize] as u16;

        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn timer_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer-=1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Implement beep sound
            }


            self.sound_timer -= 1;
        }
    }

    fn execute(&mut self, op: u16) {
        let first = (op & 0xF000) >> 12;
        let second = (op & 0x0F00) >> 8;
        let third = (op & 0x00F0) >> 4;
        let forth = op & 0x000f;

        match (first, second, third, forth) {
            (0,0,0,0) => return,
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_HEIGHT*SCREEN_WIDTH];
            },
            (0,0, 0xE, 0xE) => { // Return from subroutine
                let return_address = self.pop();
                self.pc = return_address;
            },



            (1,_, _, _) => { // Jump XXX
                let next_addr: u16 = op & 0x0FFF;
                self.pc = next_addr;
            },



            (2,_, _, _) => { // Call XXX
                let next_addr: u16 = op & 0x0FFF;
                self.push(self.pc);
                self.pc = next_addr;
            },



            (3, _, _, _) => { // If Vx == NN ==> Skip next block
                let nn =  (op&0xFF) as u8;
                if self.registers[second as usize] == nn {
                    self.pc += 2;
                }
            },



            (4, _, _, _) => { // If Vx != NN ==> Skip next block
                let nn: u8 =  (op&0xFF) as u8;
                if self.registers[second as usize] != nn {
                    self.pc += 2;
                }
            },



            (5, _, _, 0) => { // If Vx == Vy ==> Skip next block
                if self.registers[second as usize] == self.registers[third as usize] {
                    self.pc += 2;
                }
            },



            (6, _, _, _) => { // Set Vx = NN
                let nn = (op & 0xFF) as u8;
                self.registers[second as usize] = nn;
            },



            (7, _, _, _) => { // Set Vx += NN   // CF not changed
                let nn = (op & 0xFF) as u8;
                self.registers[second as usize] = self.registers[second as usize].wrapping_add(nn);
            },



            (8, _, _, 0) => { // Set Vx = Vy
                self.registers[second as usize] = self.registers[third as usize];
            },
            (8, _, _, 1) => { // Set Vx |= Vy
                self.registers[second as usize] |= self.registers[third as usize];
            },
            (8, _, _, 2) => { // Set Vx &= Vy
                self.registers[second as usize] &= self.registers[third as usize];
            },
            (8, _, _, 3) => { // Set Vx ^= Vy
                self.registers[second as usize] ^= self.registers[third as usize];
            },
            (8, _, _, 4) => { // Set Vx += Vy and change CF when overflow.
                let (new_val, carry): (u8, bool) = self.registers[second as usize].overflowing_add(self.registers[third as usize]);
                (self.registers[second as usize], self.registers[0xF]) = (new_val, if carry {1} else {0})
            },
            (8, _, _, 5) => { // Set Vx -= Vy and change CF when overflow.
                let (new_val, borrow): (u8, bool) = self.registers[second as usize].overflowing_sub(self.registers[third as usize]);
                (self.registers[second as usize], self.registers[0xF]) = (new_val, if borrow {0} else {1})
            },
            (8, _, _, 6) => { // Set SHR VX and stores the least significant bit of VX prior to the shift into VF
                self.registers[0xF] = self.registers[second as usize] & 0x1;
                self.registers[second as usize] >>= 1;
            },
            (8, _, _, 7) => { // VX = Vy - VX and set VF = VY >= VX
                let (new_val, borrow): (u8, bool) = self.registers[third as usize].overflowing_sub(self.registers[second as usize]);
                self.registers[0xF] = if borrow {0} else {1};
                self.registers[second as usize] = new_val;
            },
            (8, _, _, 0xE) => { // SHL VX 1 and set CF to most significant bit
                self.registers[0xF] = (self.registers[second as usize]>>7) & 0x1;
                self.registers[second as usize] <<= 1;
            },



            (9, _, _, 0) => { // Skips the next instruction if VX does not equal VY.
                if self.registers[second as usize] != self.registers[third as usize] {
                    self.pc += 2;
                }
            },


            
            (0xA, _, _, _) => { // Set I to NNN.
                let i_address = op & 0x0FFF;
                self.ip = i_address;
            },
            
            (0xB, _, _, _) => { // Jump to NNN then plus V0. PC = V0 + NNN	
                let new_address = self.registers[0] as u16 + (op & 0x0FFF);
                self.pc = new_address;
            },

            (0xC, _, _, _) => { // Vx = rand() & NN	
                let rand: u8 = random();
                self.registers[second as usize] = rand & (op&0x00FF) as u8;
            },

            

            (0xD, _, _, _) => { /* Draws a sprite at coordinate (VX, VY) 
                that has a width of 8 pixels and a height of N pixels. 
                Each row of 8 pixels is read as bit-coded starting from memory location I; 
                I value does not change after the execution of this instruction. 
                As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen.
                */
                let x_coord = self.registers[second as usize] as u16;
                let y_coord = self.registers[third as usize] as u16;

                let num_rows = forth;
                
                let mut bit_flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.ip + y_line;
                    let pixels = self.mem[addr as usize];

                    for x_line in 0..SPRITE_WIDTH {
                        if (pixels & (1<<SPRITE_WIDTH-1) >> x_line) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let pixel_index = x + SCREEN_WIDTH*y;
                            bit_flipped |= self.screen[pixel_index];
                            self.screen[pixel_index] ^= true

                        }
                    }
                }

                self.registers[0xF] = if (bit_flipped) {1} else {0};

            },

            (0xE, _, 9, 0xE) => { // Skip next instruction if key in VX pressed	
                if self.keys[self.registers[second as usize] as usize] {
                    self.pc += 2;
                }
            },


            (0xE, _, 0xA, 1) => { // Skip next instruction if key in VX not pressed	
                if !self.keys[self.registers[second as usize] as usize] {
                    self.pc += 2;
                }
            },



            (0xF, _, 0, 7) => { // Sets VX to the value of the delay timer.	
                self.registers[second as usize] = self.delay_timer;
            },


            (0xF, _, 0, 0xA) => { // Wait until any key pressed and store key in VX	
                let mut key_press = false;
                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        key_press = true;
                        self.registers[second as usize] = i as u8;
                        break;
                    }
                }

                if !key_press {
                    self.pc -= 2;
                }
            },


            (0xF, _, 1, 5) => { // Set delay timer = Vx
                self.delay_timer = second as u8;
            },
            (0xF, _, 1, 8) => { // Set sound timer = Vx
                self.sound_timer = second as u8;
            },


            (0xF, _, 1, 0xE) => { // Adds VX to I. VF is not affected
                self.ip = self.ip.wrapping_add(self.registers[second as usize] as u16);
            },





            (0xF, _, 2, 9) => { // Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                // Due to each character is 5 bytes long => offset = 5*VX
                self.ip = (self.registers[second as usize] as u16) * 5;
            },


            (0xF, _, 3, 3) => { // Stores the binary-coded decimal representation of VX, with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.[24]
                let number = self.registers[second as usize] as f32;
                self.mem[self.ip as usize] = (number/100.0).floor() as u8;
                self.mem[(self.ip+1) as usize] = ((number/10.0) % 10.0).floor() as u8;
                self.mem[(self.ip+2) as usize] = (number % 10.0) as u8;
            },


            (0xF, _, 5, 5) => { // Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
                for i in 0..=second {
                    self.mem[(self.ip + i) as usize] = self.registers[i as usize];
                }
            },

            (0xF, _, 6, 5) => { // Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified.
                for i in 0..=second {
                    self.registers[i as usize] = self.mem[(self.ip + i) as usize];
                }
            },

            
            
            (_, _, _, _) => unimplemented!("The op has not implemented {}", op),
        } 
    }

    pub fn get_display(&mut self) -> &[bool] {
        &self.screen
    }

    pub fn press_key(&mut self, key: usize, pressed: bool) {
        if key > 0xF {
            panic!("Key out of bound!");
        }

        self.keys[key] = pressed;
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        let end_offset = ENTRY_POINT_ADDR as usize + buffer.len();

        if end_offset > MEMORY_SIZE {
            panic!("Unable to load the rom. Memory it not fit.");
        }

        self.mem[(ENTRY_POINT_ADDR as usize)..end_offset].copy_from_slice(buffer);
    }
}