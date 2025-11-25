use cpu::v4::CPU;

fn main() {
    let mut cpu = CPU::new();

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    let mem = &mut cpu.memory;

    let call_twice = [
        0x21, 0x00, // call function at 0x100
        0x21, 0x00, // call function at 0x100
        0x00, 0x00, // end instruction
    ];
    mem[0x00..0x06].copy_from_slice(&call_twice);

    let add_twice: [u8; 6] = [
        0x80, 0x14, // add register 0 and 1
        0x80, 0x14, // add register 0 and 1
        0x00, 0xEE, // return function
    ];
    mem[0x100..0x106].copy_from_slice(&add_twice);

    cpu.run();

    // instruction is actually
    // (5 + 10)
    // (5 + 10)
    // (5 + 10)
    // (5 + 10)
    println!("5 + (10 * 2) + (10 * 2) = {}", cpu.registers[0]);

    assert_eq!(cpu.registers[0], 45);
}
