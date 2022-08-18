const WIDTH: u8 = 64;

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
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// A struct representing the CHIP-8 CPU and RAM
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    /// 4K of memory for the CHIP-8
    mem: [u8; 4096],

    /// The program counter
    pc: u16,

    /// The 'I' register to store memory addresses
    i_reg: u16,

    /// The stack for the CHIP-8
    stack: [u16; 16],

    /// The registers for the CPU
    registers: [u8; 16],

    /// The stack pointer
    sp: u8,

    /// The delay timer
    delay_timer: u8,

    /// The sound timer
    sound_timer: u8,

    /// The VF register
    vf: u8,

    /// The display buffer
    buf: [u8; 2048],
}

impl CPU {
    /// Initiate a new instance of the CPU struct
    pub fn new() -> Self {
        let mut mem: [u8; 4096] = [0; 4096];

        // Write the font to mem
        mem[0x050..(0x09F + 1)].copy_from_slice(&FONT[..]);

        CPU {
            registers: [0; 16],
            pc: 0x200,
            sp: 0,
            mem,
            stack: [0; 16],
            i_reg: 0x200,
            delay_timer: 255,
            sound_timer: 255,
            vf: 0,
            buf: [0; 2048],
        }
    }

    pub fn new_with_memory(program_memory: &[u8]) -> Self {
        let mut mem: [u8; 4096] = [0; 4096];
        // Write the font to mem
        mem[0x050..(0x09F + 1)].copy_from_slice(&FONT[..]);

        // Write the program memory to mem
        mem[0x200..(program_memory.len() + 0x200)].copy_from_slice(program_memory);

        CPU {
            registers: [0; 16],
            pc: 0x200,
            sp: 0,
            mem,
            stack: [0; 16],
            i_reg: 0x200,
            delay_timer: 255,
            sound_timer: 255,
            vf: 0,
            buf: [0; 2048],
        }
    }

    /// Decodes two bytes into 4 seperate nibbles
    pub fn decode(&self, upper_byte: u8, lower_byte: u8) -> (u8, u8, u8, u8) {
        let upper_high = ((upper_byte & 0xF0) >> 4) as u8;
        let upper_low = (upper_byte & 0x0F) as u8;

        let lower_high = ((lower_byte & 0xF0) >> 4) as u8;
        let lower_low = (lower_byte & 0x0F) as u8;

        (upper_high, upper_low, lower_high, lower_low)
    }

    /// Runs the CHIP-8
    pub fn run(&mut self) {
        loop {
            let instruction =
                self.decode(self.mem[self.pc as usize], self.mem[self.pc as usize + 1]);
            self.pc += 2;
            match instruction {
                // 0x00E0 - clr
                (0x0, 0x0, 0xE, 0x0) => {
                    self.clr00e0();
                }
                // 0x1nnn - jp
                (0x1, nnn_a, nnn_b, nnn_c) => {
                    let addr = self.to_nnn(nnn_a, nnn_b, nnn_c);
                    self.jp1nnn(addr);
                }

                // 0x6xnn - set
                (0x6, x, a, b) => {
                    let val = (a << 4) | b;
                    self.set6xnn(x, val);
                }

                // 0x7Xnn - add
                (0x7, x, a, b) => {
                    let val = (a << 4) | b;
                    self.add7xnn(x, val);
                }

                // 0xAnnn - set
                (0xA, a, b, c) => {
                    let val = self.to_nnn(a, b, c);

                    self.setannn(val);
                }

                // 0xDxyn - draw
                (0xD, x, y, n) => {
                    self.drwdxyn(x, y, n);
                }

                (0x0, 0x0, 0x0, 0x0) => {
                    break;
                }

                (a, b, c, d) => {
                    unimplemented!("0x{:x}{:x}{:x}{:x}", a, b, c, d);
                }
            }
            self.update();
        }
    }

    // ------------------------------------------------------------------
    //                            Display
    // ------------------------------------------------------------------

    /// Clears the display
    pub fn clear(&mut self) {
        self.buf.iter_mut().for_each(|i| *i = 0);
        self.update();
    }

    pub fn update(&mut self) {
        for (i, item) in self.buf.iter_mut().enumerate() {
            if i != 0 && (i - 1) % (WIDTH as usize) == 0 {
                println!();
            }

            if *item == 1 {
                print!("#");
            } else {
                print!(" ");
            }
        }
        println!();
    }

    pub fn draw(&mut self, x: u8, y: u8, n: usize) {
        let x = self.registers[x as usize] % 64;
        let y = self.registers[y as usize] % 32;
        self.vf = 0;

        let sprite = &self.mem[(self.i_reg as usize)..(self.i_reg as usize + n as usize)];

        // Convert the coordinates to an index in the frame buffer
        let mut pos_in_buf = x as usize + (WIDTH as usize * y as usize);

        // Loop through each row in the sprite

        sprite.iter().for_each(|byte_row| {
            // This loop pushes the bits one by one to the right for each iteration,
            // see if it's on or off (using the & 1) and then write it to the frame
            // buffer
            for j in (0..8).rev() {
                // The current bit value, can be 1 or 0
                let current_bit_value = (byte_row >> j) & 1;

                // Set VF to 1 if the current bit is already on and the updated bit is also on.
                // Also turn off the bit.
                if (self.buf[pos_in_buf] == 1) && (current_bit_value == 1) {
                    self.vf = 1;
                    self.buf[pos_in_buf] = 0;
                } else {
                    // Just write to the display buffer by default.
                    self.buf[pos_in_buf] = current_bit_value;
                }

                // If we reached the end of the screen, stop rendering the row
                if (pos_in_buf % (WIDTH as usize)) == 0 {
                    break;
                }

                // Incrementing X coordinate
                pos_in_buf += 1;
            }

            // Incrementing Y coordinate
            pos_in_buf += (WIDTH - 8) as usize;
        });
    }

    // ------------------------------------------------------------------
    //                         Helper functions
    // ------------------------------------------------------------------

    pub fn to_nnn(&self, a: u8, b: u8, c: u8) -> u16 {
        let byte = ((a << 4) | b) as u16;
        let byte = (byte << 4) | c as u16;

        byte
    }

    // ------------------------------------------------------------------
    //                          Instructions
    // ------------------------------------------------------------------

    /// Clear the display.
    pub fn clr00e0(&mut self) {
        self.clear();
    }

    /// Jump to location nnn.   
    pub fn jp1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// Set Vx = nn.
    pub fn set6xnn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
    }

    /// Set Vx = Vx + nn.
    pub fn add7xnn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] += nn;
    }

    /// Set I = nnn.
    pub fn setannn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF =  collision.
    pub fn drwdxyn(&mut self, x: u8, y: u8, n: u8) {
        self.draw(x, y, n as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_cpu() -> CPU {
        CPU::new()
    }

    #[test]
    fn decode_instruction_success() {
        let upper_byte = 0xAB;
        let lower_byte = 0xCD;
        let cpu = new_cpu();
        let decoded_instruction = cpu.decode(upper_byte, lower_byte);

        assert_eq!(decoded_instruction.0, 0xA);
        assert_eq!(decoded_instruction.1, 0xB);
        assert_eq!(decoded_instruction.2, 0xC);
        assert_eq!(decoded_instruction.3, 0xD);
    }

    #[test]
    fn decode_instruction_fail() {
        let upper_byte = 0xAB;
        let lower_byte = 0xCD;
        let cpu = new_cpu();
        let decoded_instruction = cpu.decode(upper_byte, lower_byte);

        assert_ne!(decoded_instruction.0, 0x6);
        assert_ne!(decoded_instruction.1, 0x9);
        assert_ne!(decoded_instruction.2, 0x4);
        assert_ne!(decoded_instruction.3, 0x2);
    }

    #[test]
    fn test_to_nnn() {
        let a = 0xA;
        let b = 0xB;
        let c = 0xC;
        let cpu = new_cpu();

        let new_val = cpu.to_nnn(a, b, c);

        assert_eq!(new_val, 0xABC);
    }

    #[test]
    fn test_jp1nnn() {
        let mut cpu = new_cpu();

        let arbitrary_value = 69;

        cpu.jp1nnn(arbitrary_value);

        assert_eq!(cpu.pc, arbitrary_value)
    }

    #[test]
    fn test_set6xnn() {
        let mut cpu = new_cpu();

        let arbitrary_value = 69;

        cpu.set6xnn(0, arbitrary_value);

        assert_eq!(cpu.registers[0], arbitrary_value);
    }

    #[test]
    fn test_add7xnn() {
        let mut cpu = new_cpu();

        let old_value = 50;

        // Previous value of register 0
        cpu.set6xnn(0, old_value);

        let added_value = 69;

        cpu.add7xnn(0, added_value);

        assert_eq!(cpu.registers[0], old_value + added_value);
    }

    #[test]
    fn test_setannn() {
        let mut cpu = new_cpu();

        let arbitrary_value = 69;

        cpu.setannn(arbitrary_value);

        assert_eq!(cpu.i_reg, arbitrary_value);
    }
}
