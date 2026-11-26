use z80_core::bus::SystemBus;
use z80_core::z80::*;

const MEMORY_SIZE: usize = 65_536;

struct TestBus {
    ram: Box<[u8; MEMORY_SIZE]>,
    io_port: Box<[u8; MEMORY_SIZE]>,
}

impl TestBus {
    fn new() -> Self {
        Self {
            ram: Box::new([0; MEMORY_SIZE]),
            io_port: Box::new([0; MEMORY_SIZE]),
        }
    }

    fn load_code(&mut self, start_addr: u16, code: &[u8]) {
        for (i, &byte) in code.iter().enumerate() {
            self.ram[(start_addr as usize) + i] = byte;
        }
    }
}

impl SystemBus for TestBus {
    fn read_memory(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write_memory(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    fn read_io(&mut self, port: u16) -> u8 {
        self.io_port[port as usize]
    }

    fn write_io(&mut self, port: u16, data: u8) {
        self.io_port[port as usize] = data;
    }
}

fn main() {
    let mut z80 = Z80::new();
    let mut bus = TestBus::new();

    let code = std::fs::read("resources/multiply_u16.bin").unwrap();
    bus.load_code(0, &code);

    let mut cycles: usize = 0;
    z80.display_regs();
    println!("");
    loop {
        cycles += z80.execute(&mut bus) as usize;
        z80.display_regs();
        println!("");
        if z80.reg.pc == 0x0019 {
            break;
        }
    }
    println!("cycles: {}", cycles);
}
