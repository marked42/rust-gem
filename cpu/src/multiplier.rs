/// executes multiple instructions pointed by program_counter until
/// instruction 0x0000 signaling end of program
pub struct CPU {
    // 16 registers
    pub registers: [u8; 16],
    pub program_counter: usize,
    // 4096 bytes RAM
    pub memory: [u8; 4096],
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        let op_high_byte = self.memory[self.program_counter] as u16;
        let op_low_byte = self.memory[self.program_counter + 1] as u16;

        op_high_byte << 8 | op_low_byte
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.program_counter += 2;

            let c = ((opcode >> 12) & 0x000F) as u8;
            let x = ((opcode >> 8) & 0x000F) as u8;
            let y = ((opcode >> 4) & 0x000F) as u8;
            let d = ((opcode >> 0) & 0x000F) as u8;

            match (c, x, y, d) {
                (0, 0, 0, 0) => return,
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }
}
