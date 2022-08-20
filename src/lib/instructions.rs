use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

use super::cpu::CPU;

impl CPU {
    /// Clear the display.
    pub fn cls00e0(&mut self) {
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

    /// Return from a subroutine.
    pub fn ret00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    /// Call subroutine at nnn.
    pub fn call2nnn(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = nnn;

        self.pc = nnn;
    }

    /// Skip next instruction if Vx = nn.
    pub fn se3xnn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] == nn {
            self.pc += 2;
        }
    }

    /// Skip next instruction if Vx != nn.
    pub fn sne4xnn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] != nn {
            self.pc += 2;
        }
    }

    /// Skip next instruction if Vx = Vy.
    pub fn se5xy0(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
    }

    /// Skip next instruction if Vx != Vy.
    pub fn sne9xy0(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    }

    /// Stores the value of register Vy in register Vx.
    pub fn ld8xy0(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    }

    /// Set Vx = Vx OR Vy.
    pub fn or8xy1(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    /// Set Vx = Vx AND Vy.
    pub fn and8xy2(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    /// Set Vx = Vx XOR Vy.
    pub fn xor8xy3(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    /// Set Vx = Vx + Vy, set VF = carry.
    pub fn add8xy4(&mut self, x: u8, y: u8) {
        // Set VF to 0
        self.vf = 0;

        // The sum casted into a u16
        let sum = self.registers[x as usize] as u16 + self.registers[y as usize] as u16;

        // If the sum overflows set VF to 1 and only write the first byte to Vx
        if sum > 255 {
            self.vf = 1;
            self.registers[x as usize] = (sum & 0xFF) as u8;
        } else {
            self.registers[x as usize] = sum as u8;
        }
    }

    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    pub fn sub8xy5(&mut self, x: u8, y: u8) {
        let difference = self.registers[x as usize] as i16 - self.registers[y as usize] as i16;
        self.vf = 0;

        if difference > 0 {
            self.vf = 1;
        }
        self.registers[x as usize] = difference as u8;
    }

    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    pub fn sub8xy7(&mut self, x: u8, y: u8) {
        let difference = self.registers[y as usize] as i16 - self.registers[x as usize] as i16;
        self.vf = 0;

        if difference > 0 {
            self.vf = 1;
        }
        self.registers[x as usize] = difference as u8;
    }

    /// Set Vx = Vy SHR 1, set VF = shifted bit
    pub fn shr8xy6_usey(&mut self, x: u8, y: u8) {
        // Get the last bit
        let last_bit = self.registers[y as usize] & 1;

        // The shifted bit
        let y_shifted = self.registers[y as usize] >> 1;

        self.vf = last_bit;
        self.registers[x as usize] = y_shifted;
    }

    /// Set Vx = Vx SHR 1, set VF = shifted bit
    pub fn shr8xy6_usex(&mut self, x: u8, _y: u8) {
        // Get the last bit
        let last_bit = self.registers[x as usize] & 1;

        // The shifted bit
        let x_shifted = self.registers[x as usize] >> 1;

        self.vf = last_bit;
        self.registers[x as usize] = x_shifted;
    }

    /// Set Vx = Vy SHL 1, set VF = shifted bit
    pub fn shl8xye_usey(&mut self, x: u8, y: u8) {
        // Get the last bit by checking if the byte is greater than 127
        let last_bit = if self.registers[y as usize] > 127 {
            1
        } else {
            0
        };

        // The shifted bit
        let y_shifted = self.registers[y as usize] << 1;

        self.vf = last_bit;
        self.registers[x as usize] = y_shifted;
    }

    /// Set Vx = Vx SHL 1, set VF = shifted bit
    pub fn shl8xye_usex(&mut self, x: u8, _y: u8) {
        // Get the last bit by checking if the byte is greater than 127
        let last_bit = if self.registers[x as usize] > 127 {
            1
        } else {
            0
        };
        // The shifted bit
        let x_shifted = self.registers[x as usize] << 1;

        self.vf = last_bit;
        self.registers[x as usize] = x_shifted;
    }

    /// Jump to location nnn + V0.
    pub fn jpbnnn(&mut self, nnn: u16) {
        self.pc = nnn + (self.registers[0] as u16);
    }

    /// Set Vx = random byte AND kk.
    pub fn rndcxnn(&mut self, x: u8, nn: u8) {
        let random_number = rand::thread_rng().gen_range(0..=255) as u8;

        self.registers[x as usize] = random_number & nn;
    }

    /// Skip next instruction if key with the value of Vx is pressed.
    #[allow(dead_code)]
    pub fn skpex9e(&mut self) {
        todo!()
    }

    /// Skip next instruction if key with the value of Vx is not pressed.
    #[allow(dead_code)]
    pub fn skpexa1(&mut self) {
        todo!()
    }

    /// Set Vx = delay timer value.
    pub fn ldfx07(&mut self, x: u8) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let then = self.last_dt_write;

        let elapsed_time_wrapped = ((now - then) % 255) as u8;

        self.registers[x as usize] = elapsed_time_wrapped;
    }

    /// Set delay timer = Vx.
    pub fn ldfx15(&mut self, x: u8) {
        self.last_dt_write = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.delay_timer = x;
    }

    /// Set sound timer = Vx.
    pub fn ldfx18(&mut self, x: u8) {
        self.last_st_write = std::time::SystemTime::now().elapsed().unwrap().as_millis();
        self.sound_timer = x;
    }

    /// Set I = I + Vx.
    pub fn addfx1e(&mut self, x: u8) {
        self.i_reg += self.registers[x as usize] as u16;
    }

    /// Set I = location of sprite for digit Vx.
    pub fn ldfx29(&mut self, x: u8) {
        self.i_reg = 0x50
            + match self.registers[x as usize] {
                0x0 => 0,
                0x1 => 5,
                0x2 => 10,
                0x3 => 15,
                0x4 => 20,
                0x5 => 25,
                0x6 => 30,
                0x7 => 35,
                0x8 => 40,
                0x9 => 45,
                0xA => 50,
                0xB => 55,
                0xC => 60,
                0xD => 65,
                0xE => 70,
                0xF => 75,
                _ => 0,
            }
    }

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    pub fn ldfx33(&mut self, x: u8) {
        let num = self.registers[x as usize];

        let mut digits: [u8; 3] = [0; 3];

        let num = num.to_string();

        for (i, char) in num.chars().enumerate() {
            digits[i] = char.to_digit(10).unwrap() as u8;
        }

        println!("{digits:?}");

        self.mem[self.i_reg as usize] = digits[0];
        self.mem[self.i_reg as usize + 1] = digits[1];
        self.mem[self.i_reg as usize + 2] = digits[2];
    }

    /// Store registers V0 through Vx in memory starting at location I.
    pub fn ldfx55(&mut self, x: u8) {
        for i in 0..(x + 1) {
            self.mem[(self.i_reg + i as u16) as usize] = self.registers[i as usize];
        }
    }

    /// Read registers V0 through Vx from memory starting at location I.
    pub fn ldfx65(&mut self, x: u8) {
        for i in 0..(x + 1) {
            self.registers[i as usize] = self.mem[(self.i_reg + i as u16) as usize];
        }
    }
}
