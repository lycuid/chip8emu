use rand;

pub type Byte = u8;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const FONTSET: [Byte; 80] = [
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

pub struct Cpu {
    memory: [Byte; 0x1000], // 4k RAM [byte; 4096].
    stack: [u16; 0x10],     // stack [2 bytes; 16].
    v: [Byte; 0x10],        // register [byte; 16].
    i: u16,                 // Index pointer [0x000..=0xFFF].
    pc: u16,                // Program counter [0x000..=0xFFF].
    sp: usize,              // Stack pointer,
    keypad: [bool; 0x10],   // key pressed [byte; 16].

    dt: Byte, // delta timer.
    st: Byte, // sound timer.

    pub frame_buffer: [[Byte; WIDTH]; HEIGHT],
    pub frame_buffer_updated: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut memory: [Byte; 0x1000] = [0; 0x1000];

        // memory[0x000..=0x1FF] Fontset.
        for (addr, &byte) in FONTSET.iter().enumerate() {
            memory[addr] = byte;
        }

        Self {
            memory,
            stack: [0; 0x10],
            v: [0; 0x10],
            i: 0,
            pc: 0x200,
            sp: 0,
            keypad: [false; 0x10],

            dt: 0,
            st: 0,

            frame_buffer: [[0; WIDTH]; HEIGHT],
            frame_buffer_updated: false,
        }
    }

    pub fn load(&mut self, rom: &[Byte; 0xFFF - 0x1FF]) {
        // memory[0x200..0xFFF] Rom.
        for (addr, &byte) in rom.iter().enumerate() {
            self.memory[addr + 0x200] = byte;
        }

        self.stack = [0; 0x10];
        self.v = [0; 0x10];
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.keypad = [false; 0x10];

        self.dt = 0;
        self.st = 0;

        self.frame_buffer = [[0; WIDTH]; HEIGHT];
        self.frame_buffer_updated = true;
    }

    pub fn emulate_cycle(&mut self) {
        // decode and handle opcode.
        self.frame_buffer_updated = false;
        let byte1 = self.memory[self.pc as usize] as u16;
        let byte2 = self.memory[self.pc as usize + 1] as u16;
        self.execute_opcode(byte1 << 8 | byte2);

        // update timer.
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // beep.
            };
            self.st -= 1;
        }
    }

    pub fn update_keypad(&mut self, keycode: char, state: bool) {
        match keycode {
            'X' => self.keypad[0] = state,
            '1' => self.keypad[1] = state,
            '2' => self.keypad[2] = state,
            '3' => self.keypad[3] = state,
            'Q' => self.keypad[4] = state,
            'W' => self.keypad[5] = state,
            'E' => self.keypad[6] = state,
            'A' => self.keypad[7] = state,
            'S' => self.keypad[8] = state,
            'D' => self.keypad[9] = state,
            'Z' => self.keypad[10] = state,
            'C' => self.keypad[11] = state,
            '4' => self.keypad[12] = state,
            'R' => self.keypad[13] = state,
            'F' => self.keypad[14] = state,
            'V' => self.keypad[15] = state,
            _ => {}
        };
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let kk = (opcode & 0x00FF) as Byte;
        let n = (opcode & 0x000F) as usize;
        let nnn = opcode & 0x0FFF;

        // println!(
        //     "opcode: {:X}\tpc: {:X}\tsp: {}\tx: {}\ty: {}\tkk: {:X}\tn: {}\taddr: {:X}",
        //     opcode, self.pc, self.sp, x, y, kk, n, nnn
        // );

        let code = (opcode & 0xF000) >> 12;
        match code {
            0x0 => match n {
                // `00E0` -> CLS
                0x0 => {
                    self.frame_buffer = [[0; WIDTH]; HEIGHT];
                    self.frame_buffer_updated = true;
                }
                // `00EE` -> RET
                0xE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                    return;
                }
                _ => std::process::exit(2),
            },

            // `1nnn` -> JP addr
            // - set program counter to `nnn`.
            0x1 => {
                self.pc = nnn;
                return;
            }

            // `2nnn` -> CALL addr
            //  - current pc goes on top the stack.
            //  - increment stack pointer.
            //  - set pc to `nnn`.
            0x2 => {
                self.stack[self.sp] = self.pc + 2;
                self.sp += 1;
                self.pc = nnn;
                return;
            }

            // `3xkk` -> SE Vx, byte
            // - skip next instruction if `Vx == kk`.
            0x3 => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }

            // `4xkk` -> SNE Vx, byte
            // - skip next instruction if `Vx != kk`.
            0x4 => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }

            // `5xy0` -> SE Vx, Vy
            // - skip next instruction if `Vx == Vy`.
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }

            // `6xkk` -> LD Vx, byte
            // - put value `kk` into register `Vx`.
            0x6 => self.v[x] = kk,

            // `7xkk` -> ADD Vx, byte
            // - set `Vx = Vx + kk`.
            0x7 => self.v[x] = self.v[x].wrapping_add(kk),

            // `8xyn`.
            0x8 => match n {
                // `8xy0` -> LD Vx, Vy (set `Vx = Vy`).
                0x0 => self.v[x] = self.v[y],

                // `8xy1` -> OR Vx, Vy (set `Vx = Vx | Vy`).
                0x1 => self.v[x] |= self.v[y],

                // `8xy2` -> AND Vx, Vy (set `Vx = Vx & Vy`).
                0x2 => self.v[x] &= self.v[y],

                // `8xy3` -> XOR Vx, Vy (set `Vx = Vx ^ Vy`).
                0x3 => self.v[x] ^= self.v[y],

                // `8xy4` -> ADD Vx, Vy
                // - calculate `Vx + Vy`
                // - set VF to 1, if `Vx + Vy` is more than 8 bits, else 0.
                // - store the lower 8 bits of `Vx + Vy` into `Vx`.
                0x4 => {
                    self.v[0xF] = (self.v[y] > (0xFF - self.v[x])) as Byte;
                    self.v[x] = self.v[x].wrapping_add(self.v[y]);
                }

                // `8xy5` -> SUB Vx, Vy
                // - set `Vx = Vx - Vy`.
                // - set VF to 1, if `Vx > Vy`, else 0.
                0x5 => {
                    self.v[0xF] = (self.v[x] > self.v[y]) as Byte;
                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                }

                // `8xy6` -> SHR Vx, Vy
                // - set `VF` to least significant bit of `Vx`
                // - divide `Vx` by 2.
                0x6 => {
                    self.v[0xF] = self.v[x] & 1;
                    self.v[x] >>= 1;
                }

                // `8xy7` -> SUBN Vx, Vy
                // - set `Vx = Vy - Vx`.
                // - set VF to 1, if `Vy > Vx`, else 0.
                0x7 => {
                    self.v[0xF] = (self.v[y] > self.v[x]) as Byte;
                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                }

                // `8xyE` -> SHL Vx, Vy
                // - set `VF` to most significant bit of `Vx`.
                // - multiply `Vx` by 2
                0xE => {
                    self.v[0xF] = self.v[x] >> 7;
                    self.v[x] <<= 1;
                }

                _ => std::process::exit(2),
            },

            // `9xy0` -> SNE Vx, Vy (skip next instruction if `Vx != Vy`).
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }

            // `Annn` -> LD I, addr (set `I` to `nnn`).
            0xA => self.i = nnn,

            // `Bnnn` -> JP V0, addr (Jump to location `nnn + V0`).
            0xB => {
                self.pc = nnn + self.v[0] as u16;
                return;
            }

            // `Cxkk` - RND Vx, byte (set `Vx` = rand AND kk).
            0xC => self.v[x] = rand::random::<Byte>() & kk,

            // `Dxyn` - DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            //
            // The interpreter reads n bytes from memory, starting at the address
            // stored in I. These bytes are then displayed as sprites on screen
            // at coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
            // If this causes any pixels to be erased, VF is set to 1, otherwise
            // it is set to 0. If the sprite is positioned so part of it is outside
            // the coordinates of the display, it wraps around to the opposite side
            // of the screen.
            0xD => {
                let vx = self.v[x] as usize;
                let vy = self.v[y] as usize;

                self.v[0xF] = 0;
                for row in 0..n {
                    let pixel = self.memory[self.i as usize + row];
                    let y = (vy + row) % HEIGHT;
                    for col in 0..8 {
                        if (pixel & (0x80 >> col)) != 0 {
                            let x = (vx + col) % WIDTH;
                            if self.frame_buffer[y][x] == 1 {
                                self.v[0xF] = 1; // collision.
                            }
                            self.frame_buffer[y][x] ^= 1;
                        }
                    }
                }
                self.frame_buffer_updated = true;
            }

            // `Exkk`.
            0xE => match kk {
                // `Ex9E`
                // skip next instruction if key with the value of Vx is pressed.
                // Checks the keyboard, and if the key corresponding to the value
                // of Vx is currently in the down position, PC is increased by 2.
                0x9E => {
                    if self.keypad[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                // `ExA1` - SKNP Vx
                // skip next instruction if key with the value of Vx is not pressed.
                0xA1 => {
                    if !self.keypad[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                _ => std::process::exit(2),
            },

            // `Fxkk`
            0xF => match kk {
                // `Fx07` -> LD Vx, DT (set `Vx` to `DT` value).
                0x07 => self.v[x] = self.dt,

                // `Fx0A` - LD Vx, K
                // - wait for a key press, store the value of the key in `Vx`.
                0x0A => {
                    if let Some(key) = self.keypad.iter().position(|&b| b) {
                        self.v[x] = key as Byte;
                    } else {
                        return;
                    }
                }

                // `Fx15` -> LD DT, Vx (set `DT` to `Vx` value).
                0x15 => self.dt = self.v[x],

                // `Fx18` - LD ST, Vx (Set `ST` to `Vx` value).
                0x18 => self.st = self.v[x],

                // `Fx1E` - ADD I, Vx (set `I = I + Vx`).
                0x1E => {
                    self.i += self.v[x] as u16;
                    self.v[0xF] = (self.i > 0x0F00) as Byte;
                }

                // `Fx29` - LD F, Vx
                // - set `I` to location of sprite for digit `Vx`.
                0x29 => {
                    self.i = self.v[x] as u16 * 5;
                }

                // `Fx33` - LD B, Vx
                // - places the hundreds digit of `Vx` at `memory[I]`.
                // - places the tens digit of `Vx` at `memory[I + 1]`.
                // - places the digits digit of `Vx` at `memory[I + 2]`.
                0x33 => {
                    let i = self.i as usize;
                    let vx = self.v[x];

                    self.memory[i] = vx / 100;
                    self.memory[i + 1] = (vx % 100) / 10;
                    self.memory[i + 2] = vx % 10;
                }

                // `Fx55` -> LD [I], Vx
                // store registers V0 through Vx in memory starting at location I.
                0x55 => {
                    for i in 0..=x {
                        self.memory[self.i as usize + i] = self.v[i];
                    }
                }

                // `Fx65` -> LD Vx, [I]
                // read registers V0 through Vx from memory starting at location I.
                0x65 => {
                    for i in 0..=x {
                        self.v[i] = self.memory[self.i as usize + i];
                    }
                }

                _ => std::process::exit(2),
            },

            _ => std::process::exit(2),
        };

        // move to the next instruction.
        self.pc += 2;
    }
}
