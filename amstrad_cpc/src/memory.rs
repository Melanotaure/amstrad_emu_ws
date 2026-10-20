use std::fs;
use std::path::Path;

const NB_RAM_OFFSETS: usize = 4;
const TOTAL_RAM_SIZE: usize = 128 * 1024;
const RAM_BLOCK_SIZE: usize = 16 * 1024;
const ROM_SIZE: usize = 16 * 1024;
const NB_UPPER_ROMS: usize = 16;

pub struct Memory {
    ram: Vec<u8>,
    lower_rom: Vec<u8>,
    upper_roms: Vec<Vec<u8>>,

    pub lower_rom_enabled: bool,
    pub upper_rom_enabled: bool,
    pub active_upper_rom: usize,

    ram_offsets: [usize; NB_RAM_OFFSETS],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: vec![0; TOTAL_RAM_SIZE],
            lower_rom: vec![0; ROM_SIZE],
            upper_roms: vec![vec![0; ROM_SIZE]; NB_UPPER_ROMS],

            lower_rom_enabled: true,
            upper_rom_enabled: true,
            active_upper_rom: 0,

            ram_offsets: [
                0 * RAM_BLOCK_SIZE,
                1 * RAM_BLOCK_SIZE,
                2 * RAM_BLOCK_SIZE,
                3 * RAM_BLOCK_SIZE,
            ],
        }
    }

    pub fn load_lower_rom<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let data = fs::read(path)?;
        if data.len() == ROM_SIZE {
            self.lower_rom.copy_from_slice(&data);
        }
        Ok(())
    }

    pub fn load_upper_rom<P: AsRef<Path>>(&mut self, slot: usize, path: P) -> std::io::Result<()> {
        let data = fs::read(path)?;
        if data.len() == ROM_SIZE && slot < NB_UPPER_ROMS {
            self.upper_roms[slot].copy_from_slice(&data);
        }
        Ok(())
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let window = (addr >> 14) as usize;
        let offset = (addr & 0x3FFF) as usize;

        // When reading, always check if it's from RAM or ROM
        if window == 0 && self.lower_rom_enabled {
            return self.lower_rom[offset];
        }

        if window == 3 && self.upper_rom_enabled {
            return self.upper_roms[self.active_upper_rom][offset];
        }

        let ram_base_index = self.ram_offsets[window];
        self.ram[ram_base_index + offset]
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        let window = (addr >> 14) as usize;
        let offset = (addr & 0x3FFF) as usize;

        // When writing, it's always in the RAM whatever the selected ROM
        let ram_base_index = self.ram_offsets[window];
        self.ram[ram_base_index + offset] = val;
    }

    /// Called by the Gate Array whenever a OUT instruction @0x7Fxx is executed
    pub fn set_ram_config(&mut self, config_value: u8) {
        // Isolate the 3 last bits
        let config = config_value & 0x07;

        // Precalculated RAM banks' blocks base adresses in Ko to ease understanding
        // RAM BANK 1: 4 x blocks (b0 -> b3)
        let b0 = 0 * RAM_BLOCK_SIZE;
        let b1 = 1 * RAM_BLOCK_SIZE;
        let b2 = 2 * RAM_BLOCK_SIZE;
        let b3 = 3 * RAM_BLOCK_SIZE;
        // RAM BANK 2: 4 x blocks (b4 -> b7)
        let b4 = 4 * RAM_BLOCK_SIZE;
        let b5 = 5 * RAM_BLOCK_SIZE;
        let b6 = 6 * RAM_BLOCK_SIZE;
        let b7 = 7 * RAM_BLOCK_SIZE;

        self.ram_offsets = match config {
            0 => [b0, b1, b2, b3], // 0xC0 : Default config
            1 => [b0, b1, b2, b7], // 0xC1
            2 => [b4, b5, b6, b7], // 0xC2
            3 => [b0, b3, b2, b3], // 0xC3
            4 => [b0, b4, b2, b3], // 0xC4 : Most used config (AMSDOS/Games)
            5 => [b0, b5, b2, b3], // 0xC5
            6 => [b0, b6, b2, b3], // 0xC6
            7 => [b0, b7, b2, b3], // 0xC7
            _ => unreachable!(),
        };
    }
}
