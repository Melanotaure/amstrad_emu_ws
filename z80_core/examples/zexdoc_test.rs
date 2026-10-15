// use std::io;
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

    let code = std::fs::read("z80_core/resources/zexdoc.cim").unwrap();
    bus.load_code(0x100, &code);

    let mut cycles: usize = 0;
    z80.reg.pc = 0x0100_u16;

    loop {
        cycles += z80.execute(&mut bus) as usize;
        if z80.reg.pc < 0x0005_u16 {
            println!("\nCPU restarted!");
            break;
        }
        if z80.n_halt == false {
            println!("\nCPU halted!");
            break;
        }

        // z80.memory_dump(0x0000, 0x22ff);
        // println!("");
        // z80.display_regs();

        // let mut input = String::new();
        // io::stdin().read_line(&mut input).unwrap();
    }
    println!("cycles: {}", cycles);
}
