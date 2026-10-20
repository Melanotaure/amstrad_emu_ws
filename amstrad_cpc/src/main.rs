use amstrad_cpc::amstrad::AmstradCpc6128;
use z80_core::z80::*;

fn main() {
    let mut amstrad = AmstradCpc6128::new();
    let mut z80 = Z80::new();

    amstrad
        .memory
        .load_lower_rom("roms/os6128.rom")
        .expect("OS error");
    amstrad
        .memory
        .load_upper_rom(0, "roms/basic6128.rom")
        .expect("BASIC error");

    amstrad
        .memory
        .load_upper_rom(7, "roms/amsdos.rom")
        .expect("AMSDOS error");

    println!("Emulator starting...");
    loop {
        z80.execute(&mut amstrad);

        // TODO: Synchronize the execution speed to not go at 5 GHZ
    }
}
