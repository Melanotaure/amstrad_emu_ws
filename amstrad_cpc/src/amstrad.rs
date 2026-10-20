use crate::memory::Memory;
use z80_core::bus::SystemBus;

pub struct AmstradCpc6128 {
    pub memory: Memory,
}

impl AmstradCpc6128 {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }
}

impl SystemBus for AmstradCpc6128 {
    // --- CLASSICAL MEMORY ROUTING ---

    fn read_memory(&mut self, addr: u16) -> u8 {
        self.memory.read_byte(addr)
    }

    fn write_memory(&mut self, addr: u16, data: u8) {
        self.memory.write_byte(addr, data);
    }

    // --- I/O PORTS ROUTING ---

    fn read_io(&mut self, port: u16) -> u8 {
        // For now, no readable component.
        // Usually, on Amstrad, a non-connected port returns 0xFF
        0xFF
    }

    fn write_io(&mut self, port: u16, data: u8) {
        // 1. Gate Array decoding (usual port @0x7Fxx)
        // Rule: bit 15 reset, bit 14 set
        if (port & 0xC000) == 0x4000 {
            match data & 0xC0 {
                0x80 => {
                    // Command %10xxxxxx: ROM config and Video Mode
                    // Bit 3 (0=Enable, 1=Disable Upper ROM)
                    // Bit 2 (0=Enable, 1=Disable Lower ROM)
                    self.memory.upper_rom_enabled = (data & 0x08) == 0;
                    self.memory.lower_rom_enabled = (data & 0x04) == 0;

                    // TODO: also send 'data' to the GateArray to manage the Video Mode
                    // self.gate_array.set_video_mode(value & 0x03);
                }
                0xC0 => {
                    // Command %11xxxxxx: 128 Ko RAM paging (0xC0 to 0xC7)
                    self.memory.set_ram_config(data);
                }
                0x00 | 0x40 => {
                    // Commands %00xxxxxx and %01xxxxxx: Palette and Colors
                    // TODO: self.gate_array.set_color(value);
                }
                _ => unreachable!(),
            }
        }
        // 2. Upper ROM selection decoding (usual port: 0xDF00)
        // Rule: bit 13 reset
        else if (port & 0x2000) == 0 {
            // Amstrad manages 16 ROM slots only
            self.memory.active_upper_rom = (data & 0x0F) as usize;
        }
    }
}
