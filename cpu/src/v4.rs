pub struct CPU {
    pub registers: [u8; 16],
    pub program_counter: usize,
    pub memory: [u8; 4096],
    pub stack: [u16; 16],
    pub stack_pointer: u8,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: [0; 16],
            program_counter: 0,
            memory: [0; 4096],
            stack: [0; 16],
            stack_pointer: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let op_byte1 = self.memory[self.program_counter] as u16;
            let op_byte2 = self.memory[self.program_counter + 1] as u16;
            let opcode = (op_byte1 << 8) | op_byte2;

            let x = ((opcode >> 8) & 0x000F) as u8;
            let y = ((opcode >> 4) & 0x000F) as u8;
            let kk = (opcode & 0x00FF) as u8;
            let op_minor = (opcode & 0x000F) as u8;

            let addr = opcode & 0x0FFF;

            self.program_counter += 2;

            match opcode {
                0x0000 => return,
                0x00E0 => { /* clear screen */ }
                0x00EE => self.ret(),
                0x1000..=0x1FFF => self.jump(addr),
                0x2000..=0x2FFF => self.call(addr),
                0x3000..=0x3FFF => self.se(x, kk),
                0x4000..=0x4FFF => self.sne(x, kk),
                0x5000..=0x5FFF => self.se(x, y),
                0x6000..=0x6FFF => self.ld(x, kk),
                0x7000..=0x7FFF => self.add(x, kk),
                0x8000..=0x8FFF => match op_minor {
                    0 => self.ld(x, self.registers[y as usize]),
                    1 => self.or_xy(x, y),
                    2 => self.and_xy(x, y),
                    3 => self.xor_xy(x, y),
                    4 => self.add_xy(x, y),
                    _ => todo!("opcode: 0x{:04X}", opcode),
                },
                _ => todo!("opcode: 0x{:04X}", opcode),
            }
        }
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize] as usize;
    }

    /// 1nnn jump to addr
    fn jump(&mut self, addr: u16) {
        self.program_counter = addr as usize;
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer as usize;
        let stack = &mut self.stack;
        if sp > stack.len() {
            panic!("Stack overflow");
        }

        stack[sp] = self.program_counter as u16;
        self.stack_pointer += 1;
        self.program_counter = addr as usize;
    }

    /// 3xxx
    /// SE Vx, byte
    /// skip if equal
    fn se(&mut self, vx: u8, kk: u8) {
        if vx == kk {
            self.program_counter += 2;
        }
    }

    /// 4xxx
    /// SNE Vx, byte
    /// skip if not equal
    fn sne(&mut self, vx: u8, kk: u8) {
        if vx != kk {
            self.program_counter += 2;
        }
    }

    /// 6xxx, LD sets the value kk into register vx
    fn ld(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    fn add(&mut self, vx: u8, kk: u8) {
        self.registers[vx as usize] += kk;
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let (val, carry) = self.registers[x as usize].overflowing_add(self.registers[y as usize]);
        self.registers[x as usize] = val;
        self.registers[0xF] = carry as u8;
    }
}
