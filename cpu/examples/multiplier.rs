use cpu::multiplier::CPU;

fn main() {
    let mut cpu = CPU {
        registers: [0; 16],
        program_counter: 0,
        memory: [0; 4096],
    };

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;
    cpu.registers[2] = 10;
    cpu.registers[3] = 10;

    let mem = &mut cpu.memory;
    // 8014 add register 0 and 1
    mem[0] = 0x80;
    mem[1] = 0x14;
    // 8024 add register 0 and 2
    mem[2] = 0x80;
    mem[3] = 0x24;
    // 8034 add register 0 and 3
    mem[4] = 0x80;
    mem[5] = 0x34;

    cpu.run();

    println!("5 + 10 + 10 + 10 = {}", cpu.registers[0]);

    assert_eq!(cpu.registers[0], 35);
}
