use z80_core::bus::SystemBus;
use z80_core::z80::*;

struct TestBus {
    ram: Vec<u8>,
}

impl SystemBus for TestBus {
    fn read_memory(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
    fn write_memory(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
    fn read_io(&mut self, _port: u16) -> u8 {
        0xFF
    }
    fn write_io(&mut self, _port: u16, _data: u8) {}
}

// 0x0100: EI ; 0x0101: HALT ; 0x0102: INC A ; 0x0103: HALT
fn setup() -> (Z80, TestBus) {
    let mut bus = TestBus {
        ram: vec![0; 65_536],
    };
    bus.ram[0x0100..0x0104].copy_from_slice(&[0xFB, 0x76, 0x3C, 0x76]);
    let mut cpu = Z80::new();
    cpu.reg.pc = 0x0100;
    cpu.reg.sp = 0xF000;
    cpu.im = InterruptMode::IM_1;
    cpu.reg.a = 0;
    (cpu, bus)
}

#[test]
fn halt_stays_halted_without_interrupt() {
    let (mut cpu, mut bus) = setup();
    cpu.execute(&mut bus); // EI
    for _ in 0..10 {
        cpu.execute(&mut bus); // HALT, then repeated NOPs
    }
    assert!(!cpu.n_halt, "HALT line must be active");
    assert_eq!(cpu.reg.pc, 0x0102, "PC must stay on the instruction after HALT");
    assert_eq!(cpu.reg.a, 0, "the instruction after HALT must not run");
}

#[test]
fn interrupt_wakes_up_halt_and_pushes_next_instruction() {
    let (mut cpu, mut bus) = setup();
    cpu.execute(&mut bus); // EI
    for _ in 0..5 {
        cpu.execute(&mut bus);
    }
    cpu.n_int = false;
    cpu.execute(&mut bus);

    assert!(cpu.n_halt, "the interrupt must release HALT");
    assert_eq!(cpu.reg.pc, 0x0038);
    assert_eq!(cpu.reg.sp, 0xEFFE);
    let ret = u16::from_le_bytes([bus.ram[0xEFFE], bus.ram[0xEFFF]]);
    assert_eq!(ret, 0x0102, "return address must be the instruction after HALT");
}
