use std::fs;
use std::io;
use std::path::Path;

pub const START_ADDR: u16 = 0x200;
pub const MEMORY_SIZE: usize = 4096;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const FONT_BASE: u16 = 0x0;

pub const FONT: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // E
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // F
];

pub struct CPU {
    pub v: [u8; 16],
    pub i: u16,
    pub stack: [u16; 16],
    pub sp: usize,
    pub pc: u16,
    pub memory: [u8; MEMORY_SIZE],
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub keyboard: [bool; 16],
    pub sound_timer: u8,
    pub delay_timer: u8,
}

impl CPU {
    pub fn new() -> Self {
        let mut memory = [0u8; 4096];
        let base_addr = FONT_BASE as usize;
        memory[base_addr..base_addr+FONT.len()].copy_from_slice(&FONT);

        Self {
            v: [0; 16],
            i: 0,
            stack: [0; 16],
            sp: 0,
            pc: START_ADDR,
            memory: memory,
            display: [false; 64 * 32],
            keyboard: [false; 16],
            sound_timer: 0,
            delay_timer: 0
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = START_ADDR as usize;
        self.memory[start..start + rom.len()].copy_from_slice(rom);
    }

    pub fn load_rom_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<usize> {
        let rom = fs::read(path)?;
        let max_len = MEMORY_SIZE - START_ADDR as usize;
        if rom.len() > max_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "ROM too large ({} bytes > {} bytes max)",
                    rom.len(),
                    max_len
                ),
            ));
        }

        let start = START_ADDR as usize;
        self.memory[start..start + rom.len()].copy_from_slice(&rom);
        Ok(rom.len())
    }

    pub fn step(&mut self) {
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[self.pc as usize + 1] as u16;
        let opcode = (hi << 8) | lo;
        self.pc += 2;
        self.execute(opcode);
    }

    pub fn execute(&mut self, opcode: u16) {
        let nnn = (opcode & 0x0FFF) as u16; // lowest 12 bits
        let n = (opcode & 0x000F) as u8; // lowest 4 bits
        let x = ((opcode & 0x0F00) >> 8) as usize; // lower 4 bits of the high byte
        let y = ((opcode & 0x00F0) >> 4) as usize; // upper 4 bits of the low byte
        let kk = (opcode & 0x00FF) as u8; // lowest 8 bits

        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode {
                0x00E0 => self.display.fill(false),
                0x00EE => {
                    self.pc = self.stack[self.sp];
                    self.sp -= 1;
                }
                _ => {}
            },
            0x1 => {
                self.pc = nnn;
            }
            0x2 => {
                self.sp += 1;
                self.stack[self.sp] = self.pc;
                self.pc = nnn;
            }
            0x3 => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2
                }
            }
            0x6 => {
                self.v[x] = kk;
            }
            0x7 => {
                self.v[x] = self.v[x].wrapping_add(kk);
            }
            0x8 => match n {
                0x0 => {
                    self.v[x] = self.v[y];
                }
                0x1 => {
                    self.v[x] = self.v[x] | self.v[y];
                }
                0x2 => {
                    self.v[x] = self.v[x] & self.v[y];
                }
                0x3 => {
                    self.v[x] = self.v[x] ^ self.v[y];
                }
                0x4 => {
                    let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = sum;
                    self.v[0x0F] = carry as u8;
                }
                0x5 => {
                    let (res, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = res;
                    self.v[0xF] = !borrow as u8;
                }
                0x6 => {
                    self.v[x] = self.v[x] >> 1;
                    self.v[0x0F] = self.v[x] & 0x1;
                }
                0x7 => {
                    let (sub, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = sub;
                    self.v[0x0F] = !borrow as u8;
                }
                0xE => {
                    self.v[x] = self.v[x] << 1;
                    self.v[0x0F] = self.v[x] >> 7;
                }
                _ => {}
            },
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => {
                self.i = nnn;
            }
            0xB => {
                self.pc = nnn + (self.v[0] as u16);
            }
            0xC => {
                let random_byte: u8 = rand::random();
                self.v[x] = random_byte & kk;
            }
            0xD => {
                let sprite: &[u8] = &self.memory[self.i as usize..(self.i as usize + n as usize)];
                let start_x = (self.v[x] % (DISPLAY_WIDTH as u8)) as usize;
                let start_y = (self.v[y] % (DISPLAY_HEIGHT as u8)) as usize;
                for row in 0..(n as usize) {
                    let py = (start_y + row) % DISPLAY_HEIGHT;
                    let sprite_byte = sprite[row];
                    for col in 0..8 {
                        let px = (start_x + col) % DISPLAY_WIDTH;
                        let sprite_pixel = (sprite_byte >> (7 - col)) & 1;
                        if sprite_pixel == 1 {
                            let screen_pixel = &mut self.display[py * DISPLAY_WIDTH + px];
                            if *screen_pixel == true {
                                self.v[0x0f] = 1;
                            }
                            *screen_pixel ^= true;
                        }
                    }
                }
            }
            0xE => match kk {
                0xE => {
                    if self.keyboard[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                0x1 => {
                    if !self.keyboard[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF => {
                match kk {
                    0x7 => {
                        self.v[x] = self.delay_timer;
                    }
                    0xA => {
                        if let Some(key) = self.keyboard.iter().position(|&k| k) {
                            self.v[x] = key as u8;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    0x15 => {
                        self.delay_timer = self.v[x];
                    }
                    0x18 => {
                        self.sound_timer = self.v[x];
                    }
                    0x1E => {
                        self.i = self.i.wrapping_add(self.v[x] as u16);
                    }
                    0x29 => {
                        let digit = self.v[x] as u16;
                        self.i = FONT_BASE + (self.v[x] & 0xF) as u16 * 5;
                    }
                    0x33 => {
                        self.memory[self.i as usize] = self.v[x] / 100;
                        self.memory[self.i as usize + 1] = (self.v[x] / 10) % 10;
                        self.memory[self.i as usize + 2] = self.v[x] % 10;
                    }
                    0x55 => {
                        for idx in 0..=x {
                            self.memory[self.i as usize + idx] = self.v[idx];
                        }
                    }
                    0x65 => {
                        for idx in 0..=x {
                            self.v[idx] = self.memory[self.i as usize + idx];
                        }
                    }
                    _ => {}
                }
            }
            _ => panic!("unimplemented opcode: {:#06x}", opcode),
        }
    }
}
