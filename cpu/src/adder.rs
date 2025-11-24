#[derive(Debug)]
pub struct CPU {
    pub current_operation: u16,
    pub registers: [u8; 2],
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        self.current_operation
    }

    pub fn run(&mut self) {
        let opcode = self.read_opcode();

        let c = ((opcode >> 12) & 0x000F) as u8;
        let x = ((opcode >> 8) & 0x000F) as u8;
        let y = ((opcode >> 4) & 0x000F) as u8;
        let d = ((opcode >> 0) & 0x000F) as u8;

        match (c, x, y, d) {
            (0x8, _, _, 0x4) => self.add_xy(x, y),
            _ => todo!("opcode {:04x}", opcode),
        }
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] += self.registers[y as usize]
    }
}
