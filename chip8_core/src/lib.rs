use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const RAM_SIZE: usize = 4096;
const NUM_KEYS: usize = 16;
const NUM_REGS: usize = 16;
const START_ADDR: u16 = 0x200;
const STACK_SIZE: usize = 16;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH*SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        return new_emu;
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH*SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        // Fetch
        let instruction = self.fetch();
        // Decode & Execute
        self.execute(instruction)
    }

    pub fn tick_timers(&mut self) -> bool {
        let mut beep = false;
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                beep = true;
            }
            self.st -= 1;
        }

        return beep;
    }

    pub fn get_display(&self) -> &[bool] {
        return &self.screen;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();

        self.ram[start..end].copy_from_slice(data);
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // NOP
            (0, 0, 0xE, 0) => { // Clear Screen
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            (0, 0, 0xE, 0xE) => { // Return from subroutine
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            (1, _, _, _) => { // Jump to address 0xNNN
                let addr = op & 0x0FFF;
                self.pc = addr;
            },
            (2, _, _, _) => { // Enter subroutine
                let nnn = op & 0x0FFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            (3, _, _, _) => { // Skip next instruction if Vx == KK
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;
                if self.v_reg[x] == kk {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => { // Skip next instruction if Vx != KK
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;
                if self.v_reg[x] != kk {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => { // Skip next instruction if Vx == Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => { // Set Vx to NN
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = nn;
            },
            (7, _, _, _) => { // Add NN to Vx
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            (8, _, _, 0) => { // Set Vx = Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            (8, _, _, 1) => { // Set Vx = Vx OR Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            (8, _, _, 2) => { // Set Vx = Vx AND Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            (8, _, _, 3) => { // Set Vx = Vx XOR Vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            (8, _, _, 4) => { // Set Vx = Vx + Vy, set VF = carry
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 5) => { // Set Vx = Vx - Vy, set VF = NOT borrow.
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 6) => { // Set Vx = Vx SHR 1
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 0x01;

                self.v_reg[x] >>= 1; // divide by 2 shifting 1 bit right
                self.v_reg[0xF] = lsb;
            },
            (8, _, _, 7) => { // Set Vx = Vy - Vx, set VF = NOT borrow.
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 0xE) => { // Set Vx = Vx SHL 1.
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;

                self.v_reg[0xF] = msb;
                self.v_reg[x] <<= 1; // multiply by 2
            },
            (9, _, _, 0) => { // Skip next instruction if Vx != Vy.
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc +=2;
                }
            },
            (0xA, _, _, _) => { // Set I = nnn.
                let nn = op & 0x0FFF;
                self.i_reg = nn;
            },
            (0xB, _, _, _) => { // Jump to location nnn + V0.
                let nn = (op & 0x0FFF) as u16;
                self.pc = (self.v_reg[0] as u16) + nn;
            },
            (0xC, _, _, _) => { // Set Vx = random byte AND kk.
                let x = digit2 as usize;
                let kk = (op & 0x00FF) as u8;

                let rng: u8 = random();
                self.v_reg[x] = rng & kk;
            },
            (0xD, _, _, _) => { // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let num_rows = digit4;
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Calculate final position on screen after wrapping
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // calculate idx in our 1D emu screen from chip8 2D screen
                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            (0xE, _, 9, 0xE) => { // Skip next instruction if key with the value of Vx is pressed
                let x = digit2 as usize;
                let key_index = self.v_reg[x] as usize;

                if self.keys[key_index] {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => { // Skip next instruction if key with the value of Vx is not pressed.
                let x = digit2 as usize;
                let key_index = self.v_reg[x] as usize;

                if !self.keys[key_index] {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => { // Set Vx = delay timer value.
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            (0xF, _, 0, 0xA) => { // Wait for a key press, store the value of the key in Vx.
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 1, 5) => { // Set delay timer = Vx.
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            (0xF, _, 1, 8) => { // Set sound timer = Vx.
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            (0xF, _, 1, 0xE) => { // Set I = I + Vx.
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            (0xF, _, 2, 9) => { // Set I = location of sprite for digit Vx
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            (0xF, _, 3, 3) => { // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                // TODO: Reimplement with more efficient algo
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            (0xF, _, 5, 5) => { // Store registers V0 through Vx in memory starting at location I.
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for index in 0..=x {
                    self.ram[i + index] = self.v_reg[index];
                }
            },
            (0xF, _, 6, 5) => { // Read registers V0 through Vx from memory starting at location I.
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for index in 0..=x {
                    self.v_reg[index] = self.ram[i + index]
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented instruction: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        let hb = self.ram[self.pc as usize] as u16;
        let lb = self.ram[(self.pc + 1) as usize] as u16;
        let op = (hb << 8) | lb;
        self.pc += 2;
        return op;
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}