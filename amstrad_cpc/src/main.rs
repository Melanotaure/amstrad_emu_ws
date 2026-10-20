use std::thread;
use std::time::{Duration, Instant};

use amstrad_cpc::amstrad::AmstradCpc6128;
use z80_core::z80::*;

const CYCLES_PER_FRAME: u32 = 80_000; // 4 MHz / 50 Hz (Image Rate for PAL standard)
const FRAME_DURATION_MS: u64 = 20; // 1000 ms / 50 fps = 20 ms

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

    let mut cycles_this_frame = 0;
    let mut frame_start = Instant::now();
    let frame_target_duration = Duration::from_millis(FRAME_DURATION_MS);

    println!("Emulator starting @4 MHz...");

    loop {
        let cycles = z80.execute(&mut amstrad);
        cycles_this_frame += cycles as u32;

        if cycles_this_frame >= CYCLES_PER_FRAME {
            let elapsed = frame_start.elapsed();

            if elapsed < frame_target_duration {
                thread::sleep(frame_target_duration - elapsed);
            }

            cycles_this_frame -= CYCLES_PER_FRAME;

            frame_start += frame_target_duration;

            // Render the screen here
            // amstrad.crtc.render_frame();
        }
    }
}
