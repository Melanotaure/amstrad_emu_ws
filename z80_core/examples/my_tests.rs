use std::io;
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

    // Code
    // LD HL, 0x0300
    bus.write_memory(0x0100, 0x21);
    bus.write_memory(0x0101, 0x00);
    bus.write_memory(0x0102, 0x03);
    // LD SP, HL
    bus.write_memory(0x0103, 0xF9);
    // LD HL, 0x0000
    bus.write_memory(0x0104, 0x21);
    bus.write_memory(0x0105, 0x00);
    bus.write_memory(0x0106, 0x00);
    // PUSH HL
    bus.write_memory(0x0107, 0xE5);
    // POP AF
    bus.write_memory(0x0108, 0xF1);
    // LD BC, 0x1234
    bus.write_memory(0x0109, 0x01);
    bus.write_memory(0x010A, 0x34);
    bus.write_memory(0x010B, 0x12);
    // ADC HL, BC
    bus.write_memory(0x010C, 0xED);
    bus.write_memory(0x010D, 0x4A);
    // JP C, 0x0200 (error)
    bus.write_memory(0x010E, 0xDA);
    bus.write_memory(0x010F, 0x00);
    bus.write_memory(0x0110, 0x02);
    // SBC HL, BC
    bus.write_memory(0x0111, 0xED);
    bus.write_memory(0x0112, 0x42);
    // PUSH AF
    bus.write_memory(0x0113, 0xF5);
    // POP DE
    bus.write_memory(0x0114, 0xD1);
    // LD A, E
    bus.write_memory(0x0115, 0x7B);
    // CP 0x52 (expected flags)
    bus.write_memory(0x0116, 0xFE);
    bus.write_memory(0x0117, 0x52);
    // JP NZ, 0x0200 (error)
    bus.write_memory(0x0118, 0xC2);
    bus.write_memory(0x0119, 0x00);
    bus.write_memory(0x011A, 0x02);
    // INC HL
    bus.write_memory(0x011B, 0x23);
    // PUSH HL
    bus.write_memory(0x011C, 0xE5);
    // POP AF
    bus.write_memory(0x011D, 0xF1);
    // LD DE, 0xFFFF
    bus.write_memory(0x011E, 0x11);
    bus.write_memory(0x011F, 0xFE);
    bus.write_memory(0x0120, 0xFF);
    // ADC HL, DE
    bus.write_memory(0x0121, 0xED);
    bus.write_memory(0x0122, 0x5A);
    // PUSH AF
    bus.write_memory(0x0123, 0xF5);
    // POP BC
    bus.write_memory(0x0124, 0xC1);
    // LD A, C
    bus.write_memory(0x0125, 0x79);
    // CP 0x51
    bus.write_memory(0x0126, 0xFE);
    bus.write_memory(0x0127, 0x51);
    // JP NZ, 0x0200 (erreur)
    bus.write_memory(0x0128, 0xC2);
    bus.write_memory(0x0129, 0x00);
    bus.write_memory(0x012A, 0x02);
    // SBC HL, DE
    bus.write_memory(0x012B, 0xED);
    bus.write_memory(0x012C, 0x52);
    // PUSH AF
    bus.write_memory(0x012D, 0xF5);
    // POP BC
    bus.write_memory(0x012E, 0xC1);
    // LD A, C
    bus.write_memory(0x012F, 0x79);
    // CP 0x55
    bus.write_memory(0x0130, 0xFE);
    bus.write_memory(0x0131, 0x55);
    // JP NZ, 0x0200 (erreur)
    bus.write_memory(0x0132, 0xC2);
    bus.write_memory(0x0133, 0x00);
    bus.write_memory(0x0134, 0x02);
    // CALL 0x0000
    bus.write_memory(0x0135, 0xCD);
    bus.write_memory(0x0136, 0x00);
    bus.write_memory(0x0137, 0x00);
    // error: HALT
    bus.write_memory(0x0200, 0x76);

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

        z80.memory_dump(0x0000, 0x0310, &mut bus);
        println!("");
        z80.display_regs();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }
    println!("cycles: {}", cycles);
}
