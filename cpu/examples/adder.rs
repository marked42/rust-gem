use cpu::adder::CPU;

fn main() {
    let mut cpu = CPU {
        current_operation: 0,
        registers: [0; 2],
    };

    // 8 - involves two registers
    // 0 - register[0]
    // 1 - register[1]
    // 4 - addition
    // add register 0 and register 1, store sum in register1
    cpu.current_operation = 0x8014;
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    cpu.run();

    println!("cpu: {:?}", cpu)
}
