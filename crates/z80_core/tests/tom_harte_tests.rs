use serde::Deserialize;
use std::fs;
use std::path::Path;
use z80_core::bus::SystemBus;
use z80_core::z80::{InterruptMode, Z80};

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

    fn _load_code(&mut self, start_addr: u16, code: &[u8]) {
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

// JSon structure
#[derive(Deserialize, Debug)]
struct Z80State {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    // Extra registers
    af_: u16,
    bc_: u16,
    de_: u16,
    hl_: u16,
    ix: u16,
    iy: u16,
    i: u8,
    r: u8,
    iff1: u8,
    iff2: u8,
    im: u8,
    q: u8,
    // wz hidden internal register "MEMPTR"
    // #[serde(default)] if the register is not in the JSON
    #[serde(default)]
    wz: u16,
    ram: Vec<(u16, u8)>,
}

#[derive(Deserialize, Debug)]
struct Z80Test {
    name: String,
    initial: Z80State,
    #[serde(rename = "final")]
    final_state: Z80State,
}

/// This function takes one JSON test and executes it through the Z80 emulator
fn run_single_test(test: &Z80Test) {
    if test.name.starts_with("DB")
        || test.name.starts_with("DD DB")
        || test.name.starts_with("FD DB")
        || test.name.starts_with("ED 40")
        || test.name.starts_with("ED 48")
        || test.name.starts_with("ED 50")
        || test.name.starts_with("ED 58")
        || test.name.starts_with("ED 60")
        || test.name.starts_with("ED 68")
        || test.name.starts_with("ED 70")
        || test.name.starts_with("ED 78")
        || test.name.starts_with("ED A2")
        || test.name.starts_with("ED B2")
        || test.name.starts_with("ED AA")
        || test.name.starts_with("ED BA")
        || test.name.starts_with("ED B3")
        || test.name.starts_with("ED BB")
    {
        // Bypass the IN tests...
        return;
    }
    let mut cpu = Z80::new();
    let mut bus = TestBus::new();

    let init = &test.initial;
    cpu.reg.pc = init.pc;
    cpu.reg.sp = init.sp;
    cpu.reg.a = init.a;
    cpu.reg.b = init.b;
    cpu.reg.c = init.c;
    cpu.reg.d = init.d;
    cpu.reg.e = init.e;
    cpu.reg.flags.from_byte(init.f);
    cpu.reg.h = init.h;
    cpu.reg.l = init.l;

    cpu.reg.eaf = init.af_;
    cpu.reg.ebc = init.bc_;
    cpu.reg.ede = init.de_;
    cpu.reg.ehl = init.hl_;
    cpu.reg.wz = init.wz;

    cpu.reg.set_ix(init.ix);
    cpu.reg.set_iy(init.iy);
    cpu.reg.i = init.i;
    cpu.reg.r = init.r;

    cpu.iff1 = init.iff1 != 0;
    cpu.iff2 = init.iff2 != 0;
    cpu.im = match init.im {
        0 => InterruptMode::IM_0,
        1 => InterruptMode::IM_1,
        2 => InterruptMode::IM_2,
        _ => InterruptMode::IM_0,
    };
    cpu.reg.flags.alu = init.q != 0;

    for &(addr, val) in &init.ram {
        bus.write_memory(addr, val);
    }

    let _cycles = cpu.execute(&mut bus);
    if cpu.p_inst == 0xDD || cpu.p_inst == 0xFD {
        cpu.execute(&mut bus);
    }

    let fin = &test.final_state;

    assert_eq!(cpu.reg.pc, fin.pc, "[{}] PC error", test.name);
    assert_eq!(cpu.reg.sp, fin.sp, "[{}] SP error", test.name);
    assert_eq!(cpu.reg.a, fin.a, "[{}] Reg A error", test.name);
    assert_eq!(cpu.reg.b, fin.b, "[{}] Reg B error", test.name);
    assert_eq!(cpu.reg.c, fin.c, "[{}] Reg C error", test.name);
    assert_eq!(cpu.reg.d, fin.d, "[{}] Reg D error", test.name);
    assert_eq!(cpu.reg.e, fin.e, "[{}] Reg E error", test.name);
    assert_eq!(
        cpu.reg.flags.to_byte(),
        fin.f,
        "[{}] Reg F error {}",
        test.name,
        cpu.reg.pc
    );
    assert_eq!(cpu.reg.h, fin.h, "[{}] Reg H error", test.name);
    assert_eq!(cpu.reg.l, fin.l, "[{}] Reg L error", test.name);
    assert_eq!(
        cpu.reg.flags.alu,
        fin.q != 0,
        "[{}] Reg Q error {}",
        test.name,
        cpu.reg.flags.to_byte()
    );

    for &(addr, val) in &fin.ram {
        let actual_val = bus.read_memory(addr);
        assert_eq!(actual_val, val, "[{}] RAM error @{:04X}", test.name, addr);
    }
}

#[test]
fn test_tom_harte_z80() {
    let folder_path = Path::new("./tests/testdata/z80/v1");

    for entry in fs::read_dir(folder_path).expect("Can't find Tests folder") {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            println!(
                "Execution of the test suite: {:?}",
                path.file_name().unwrap()
            );

            let data = fs::read_to_string(&path).expect("Can't read the file.");

            let tests: Vec<Z80Test> = serde_json::from_str(&data)
                .unwrap_or_else(|e| panic!("Parsing error in {:?}: {}", path, e));

            for test in tests {
                run_single_test(&test);
            }
        }
    }
}
