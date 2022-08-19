const USE_NEW_SHIFTING_CONVENTIONS: bool = false;

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
    pub mem: [u8; 4096],

    /// The program counter
    pub pc: u16,

    /// The 'I' register to store memory addresses
    pub i_reg: u16,

    /// The stack for the CHIP-8
    pub stack: [u16; 16],

    /// The registers for the CPU
    pub registers: [u8; 16],

    /// The stack pointer
    pub sp: u8,

    /// The delay timer
    pub delay_timer: u8,

    /// The sound timer
    pub sound_timer: u8,

    /// The VF register
    pub vf: u8,

    /// The display buffer
    pub buf: [u8; 2048],

    // Variables for helping with internals, not meant for instruction use.
    pub last_st_write: u128,
    pub last_dt_write: u128,
}

impl CPU {
    /// Initiate a new instance of the CPU struct

    #[allow(dead_code)]
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
            last_st_write: 0,
            last_dt_write: 0,
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
            last_st_write: 0,
            last_dt_write: 0,
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
                    self.cls00e0();
                }
                // 0x1nnn - jp
                (0x1, nnn_a, nnn_b, nnn_c) => {
                    let addr = self.to_nnn(nnn_a, nnn_b, nnn_c);
                    self.jp1nnn(addr);
                }

                // 0x6xnn - set
                (0x6, x, upper_nibble, lower_nibble) => {
                    let nn = (upper_nibble << 4) | lower_nibble;
                    self.set6xnn(x, nn);
                }

                // 0x7Xnn - add
                (0x7, x, upper_nibble, lower_nibble) => {
                    let nn = (upper_nibble << 4) | lower_nibble;
                    self.add7xnn(x, nn);
                }

                // 0xAnnn - set
                (0xA, nnn_a, nnn_b, nnn_c) => {
                    let nn = self.to_nnn(nnn_a, nnn_b, nnn_c);

                    self.setannn(nn);
                }

                // 0xDxyn - draw
                (0xD, x, y, n) => {
                    self.drwdxyn(x, y, n);
                }

                // 0x2nnn - call
                (0x2, nnn_a, nnn_b, nnn_c) => {
                    let addr = self.to_nnn(nnn_a, nnn_b, nnn_c);
                    self.call2nnn(addr);
                }

                // 0x00EE - return
                (0x0, 0x0, 0xE, 0xE) => {
                    self.ret00ee();
                }

                // 0x3xnn - se
                (0x3, x, upper_nibble, lower_nibble) => {
                    let nn = (upper_nibble << 4) | lower_nibble;
                    self.se3xnn(x, nn);
                }

                // 0x4xnn - sne
                (0x4, x, upper_nibble, lower_nibble) => {
                    let nn = (upper_nibble << 4) | lower_nibble;
                    self.sne4xnn(x, nn);
                }

                // 0x5xy0 - se
                (0x5, x, y, 0x0) => {
                    self.se5xy0(x, y);
                }

                // 0x9xy0 - sne
                (0x9, x, y, 0x0) => {
                    self.sne9xy0(x, y);
                }

                // 0x8xy0 - ld
                (0x8, x, y, 0x0) => {
                    self.ld8xy0(x, y);
                }

                // 0x8xy1 - bitwise OR
                (0x8, x, y, 0x1) => {
                    self.or8xy1(x, y);
                }

                // 0x8xy2 - bitwise AND
                (0x8, x, y, 0x2) => {
                    self.and8xy2(x, y);
                }

                // 0x8xy3 - bitwise XOR
                (0x8, x, y, 0x3) => {
                    self.xor8xy3(x, y);
                }

                // 0x8xy4 - ADD
                (0x8, x, y, 0x4) => {
                    self.add8xy4(x, y);
                }

                // 0x8xy5 - SUB
                (0x8, x, y, 0x5) => {
                    self.sub8xy5(x, y);
                }

                // 0x8xy7 - SUB
                (0x8, x, y, 0x7) => {
                    self.sub8xy7(x, y);
                }

                // 0x8xy6 - shr
                (0x8, x, y, 0x6) => match USE_NEW_SHIFTING_CONVENTIONS {
                    true => {
                        self.shr8xy6_usex(x, y);
                    }
                    false => {
                        self.shr8xy6_usey(x, y);
                    }
                },

                // 0x8xy6 - shr
                (0x8, x, y, 0xE) => match USE_NEW_SHIFTING_CONVENTIONS {
                    true => {
                        self.shl8xye_usex(x, y);
                    }
                    false => {
                        self.shl8xye_usey(x, y);
                    }
                },

                (0xB, nnn_a, nnn_b, nnn_c) => {
                    let nnn = self.to_nnn(nnn_a, nnn_b, nnn_c);

                    self.jpbnnn(nnn);
                }

                (0xC, x, upper_nibble, lower_nibble) => {
                    let nn = (upper_nibble << 4) | lower_nibble;

                    self.rndcxnn(x, nn);
                }

                (0xF, x, 0x0, 0x7) => {
                    self.ldfx07(x);
                }

                (0xF, x, 0x1, 0x5) => {
                    self.ldfx15(x);
                }

                (0xF, x, 0x1, 0x8) => {
                    self.ldfx18(x);
                }

                (a, b, c, d) => {
                    unimplemented!("0x{:x}{:x}{:x}{:x}", a, b, c, d);
                }
            }
            self.update();
        }
    }

    // ------------------------------------------------------------------
    //                         Helper functions
    // ------------------------------------------------------------------

    pub fn to_nnn(&self, a: u8, b: u8, c: u8) -> u16 {
        let mut byte = ((a << 4) | b) as u16;
        byte = (byte << 4) | c as u16;

        byte
    }

    // ------------------------------------------------------------------
    //                          Instructions
    // ------------------------------------------------------------------
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

    #[test]
    fn test_call2nnn() {
        let mut cpu = new_cpu();

        let arbitrary_address = 500;

        cpu.call2nnn(arbitrary_address);

        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.pc, arbitrary_address);
        assert_eq!(cpu.stack[cpu.sp as usize], arbitrary_address);
    }

    #[test]
    fn test_se3xnn() {
        let mut cpu = new_cpu();

        let arbitrary_value = 69;

        cpu.set6xnn(0, arbitrary_value);

        // Set the PC to a random address
        cpu.pc = 500;

        // This should increment cpu.pc by 2.
        cpu.se3xnn(0, arbitrary_value);

        assert_eq!(cpu.pc, 502);
    }

    #[test]
    fn call_and_return_subroutine() {
        let mut cpu = new_cpu();

        let arbitrary_subroutine_address = 500;

        cpu.call2nnn(arbitrary_subroutine_address);
        cpu.ret00ee();

        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_sne4xnn() {
        let mut cpu = new_cpu();

        let arbitrary_value = 69;

        cpu.set6xnn(0, arbitrary_value);

        // Set the PC to a random address
        cpu.pc = 500;

        // This should increment cpu.pc by 2.
        cpu.sne4xnn(0, 42);

        assert_eq!(cpu.pc, 502);
    }

    #[test]
    fn test_se5xy0() {
        let mut cpu = new_cpu();

        let x_val = 69;
        let y_val = 69;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.pc = 500;

        cpu.se5xy0(0, 1);

        assert_eq!(cpu.pc, 502);
    }

    #[test]
    fn test_sne9xy0() {
        let mut cpu = new_cpu();

        let x_val = 69;
        let y_val = 42;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.pc = 500;

        cpu.sne9xy0(0, 1);

        assert_eq!(cpu.pc, 502);
    }

    #[test]
    fn test_ld8xy0() {
        let mut cpu = new_cpu();

        let x_val = 69;
        let y_val = 42;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.ld8xy0(0, 1);

        assert_eq!(cpu.registers[0], cpu.registers[1])
    }

    #[test]
    fn test_or8xy1() {
        let mut cpu = new_cpu();

        let x_val = 0b1000101;
        let y_val = 0b101010;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.or8xy1(0, 1);

        assert_eq!(cpu.registers[0], x_val | y_val);
    }

    #[test]
    fn test_and8xy2() {
        let mut cpu = new_cpu();

        let x_val = 0b1000101;
        let y_val = 0b101010;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.and8xy2(0, 1);

        assert_eq!(cpu.registers[0], x_val & y_val);
    }

    #[test]
    fn test_xor8xy3() {
        let mut cpu = new_cpu();

        let x_val = 0b1000101;
        let y_val = 0b101010;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.xor8xy3(0, 1);

        assert_eq!(cpu.registers[0], x_val ^ y_val);
    }

    #[test]
    fn test_add8xy4_without_overflow() {
        let mut cpu = new_cpu();

        let x_val = 10;
        let y_val = 10;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.add8xy4(0, 1);

        assert_eq!(cpu.vf, 0);
        assert_eq!(cpu.registers[0], 20);
    }

    #[test]
    fn test_add8xy4_with_overflow() {
        let mut cpu = new_cpu();

        let x_val = 255;
        let y_val = 255;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.add8xy4(0, 1);

        assert_eq!(cpu.vf, 1);
    }

    #[test]
    fn test_sub8xy5_without_underflow() {
        let mut cpu = new_cpu();

        let x_val = 50;
        let y_val = 25;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.sub8xy5(0, 1);

        assert_eq!(cpu.vf, 1);
        assert_eq!(cpu.registers[0], 25);
    }

    #[test]
    fn test_sub8xy5_with_underflow() {
        let mut cpu = new_cpu();

        let x_val = 25;
        let y_val = 50;

        cpu.set6xnn(0, x_val);
        cpu.set6xnn(1, y_val);

        cpu.sub8xy5(0, 1);

        assert_eq!(cpu.vf, 0);
        assert_eq!(cpu.registers[0], 231);
    }
}
