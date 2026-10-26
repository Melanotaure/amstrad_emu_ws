use crate::crtc_6845::Crtc;
use crate::gate_array::GateArray;
use crate::memory::Memory;
use crate::ppi_8255::Ppi8255;
use crate::psg_ay::PsgAy;
use crate::video::{Framebuffer, SCREEN_WIDTH};
use z80_core::bus::SystemBus;

pub struct AmstradCpc6128 {
    pub memory: Memory,
    pub crtc: Crtc,
    pub gate_array: GateArray,
    pub framebuffer: Framebuffer,
    pub ppi: Ppi8255,
    pub current_vsync: bool, // Updated at each video cycle
    pub psg: PsgAy,
}

impl AmstradCpc6128 {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            crtc: Crtc::new(),
            gate_array: GateArray::new(),
            framebuffer: Framebuffer::new(),
            ppi: Ppi8255::new(),
            current_vsync: false,
            psg: PsgAy::new(),
        }
    }

    pub fn step_video(&mut self) {
        let crtc_output = self.crtc.clock();

        if crtc_output.hsync {
            self.framebuffer.beam_y += 1;
            self.framebuffer.beam_x = 0;

            // Interruption Logic
        }

        if crtc_output.display_enable {
            let ram_address = self.calculate_video_address(crtc_output.ma, crtc_output.ra);
            let byte_even = self.memory.read_byte_direct(ram_address);
            let byte_odd = self.memory.read_byte_direct(ram_address + 1);
            let pens_even = self.gate_array.decode_byte(byte_even);
            let pens_odd = self.gate_array.decode_byte(byte_odd);

            let mut all_pens = [0_u8; 16];
            let len_even = pens_even.len();
            all_pens[..len_even].copy_from_slice(pens_even);
            all_pens[len_even..(len_even + pens_odd.len())].copy_from_slice(pens_odd);

            self.framebuffer.push_16_pixels(
                self.gate_array.video_mode,
                &all_pens,
                &self.gate_array.pens_rgb,
            );
        } else {
            let border_color = self.gate_array.border_rgb;
            for i in 0..16 {
                let idx = self.framebuffer.beam_y * SCREEN_WIDTH + self.framebuffer.beam_x + i;
                if idx < self.framebuffer.pixels.len() {
                    self.framebuffer.pixels[idx] = border_color;
                }
            }
        }
        self.current_vsync = crtc_output.vsync;
        self.framebuffer.beam_x += 16;
    }

    #[inline(always)]
    pub fn calculate_video_address(&self, ma: u16, ra: u8) -> u16 {
        let base = (ma & 0x3000) << 2;
        let line = ((ra as u16) & 0x07) << 11;
        let column = (ma & 0x03FF) << 1;

        base | line | column
    }

    fn sync_psg(&mut self) {
        let bdir = (self.ppi.port_c & 0x80) != 0; // bit 7
        let bc1 = (self.ppi.port_c & 0x40) != 0; // bit 6
        let line = (self.ppi.port_c & 0x0F) as usize; // Bits 0..3 (Keyboard lines)

        let psg_output = self.psg.update_bus(bdir, bc1, self.ppi.port_a_out, line);

        // If the PSG emits a piece of data (reading), it arrives on the PPI port A
        if let Some(data) = psg_output {
            self.ppi.port_a_in = data;
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
        // PPI decoding: A11 = 0
        if (port & 0x0800) == 0 {
            return self.ppi.read_io(port, self.current_vsync);
        }

        // CRTC decoding: A14 = 0
        if (port & 0x4000) == 0 {
            // Read CRTC registers if necessary
            return 0xFF;
        }

        // Usually, on Amstrad, a non-connected port returns 0xFF
        0xFF
    }

    fn write_io(&mut self, port: u16, data: u8) {
        // 1. Gate Array decoding (usual port @0x7Fxx)
        // Rule: A15 = 0,  A14 = 1
        if (port & 0xC000) == 0x4000 {
            match data & 0xC0 {
                0x00 => {
                    // %00xxxxxx: Select pen
                    self.gate_array.select_pen(data);
                }
                0x40 => {
                    // %10xxxxxx: apply the color
                    self.gate_array.set_color(data);
                }
                0x80 => {
                    // Command %10xxxxxx: ROM config and Video Mode
                    // Bit 3 (0=Enable, 1=Disable Upper ROM)
                    // Bit 2 (0=Enable, 1=Disable Lower ROM)
                    self.memory.upper_rom_enabled = (data & 0x08) == 0;
                    self.memory.lower_rom_enabled = (data & 0x04) == 0;
                    self.gate_array.set_video_mode(data & 0x03);

                    // Bit 4: manual reset of the interrupt counter
                    if (data & 0x10) != 0 {
                        self.gate_array.clear_interrupt_counter();
                    }
                }
                0xC0 => {
                    // Command %11xxxxxx: 128 Ko RAM paging (0xC0 to 0xC7)
                    self.memory.set_ram_config(data);
                }
                _ => unreachable!(),
            }
        }
        // 2. Upper ROM selection decoding (usual port: 0xDF00)
        // Rule: A13 = 0
        if (port & 0x2000) == 0 {
            // Amstrad manages 16 ROM slots only
            self.memory.active_upper_rom = (data & 0x0F) as usize;
        }
        // 3. CRTC 6845: A14 = 0 (usual ports: 0xBCxx and 0xBDxx)
        if (port & 0x4000) == 0 {
            self.crtc.write_io(port, data);
        }
        // 4. PPI 8255: A11 = 0
        if (port & 0x0800) == 0 {
            self.ppi.write_io(port, data);
            self.sync_psg();
        }
    }
}
