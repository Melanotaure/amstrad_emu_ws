use crate::crtc_6845::Crtc;
use crate::gate_array::GateArray;
use crate::memory::Memory;
use crate::ppi_8255::Ppi8255;
use crate::psg_ay::PsgAy;
use crate::video::Framebuffer;
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
    pub fn new(sample_rate: f32) -> Self {
        Self {
            memory: Memory::new(),
            crtc: Crtc::new(),
            gate_array: GateArray::new(),
            framebuffer: Framebuffer::new(),
            ppi: Ppi8255::new(),
            current_vsync: false,
            psg: PsgAy::new(sample_rate),
        }
    }

    pub fn step_video(&mut self) {
        // 1. Le CRTC avance d'1 µs
        let crtc_output = self.crtc.clock();
        self.current_vsync = crtc_output.vsync;

        // 2. Mise à jour du compteur d'interruptions du Gate Array
        self.gate_array
            .update_sync(crtc_output.hsync, crtc_output.vsync);

        // 3. Synchronisation de la position X/Y du faisceau sur le moniteur
        self.framebuffer
            .sync_beam(crtc_output.hsync, crtc_output.vsync);

        // 4. Dessin de la zone active ou de la bordure
        if crtc_output.display_enable {
            let addr = self.calculate_video_address(crtc_output.ma, crtc_output.ra);
            let byte1 = self.memory.read_byte_direct(addr);
            let byte2 = self.memory.read_byte_direct(addr.wrapping_add(1));

            let pens1 = self.gate_array.decode_byte(byte1);
            let pens2 = self.gate_array.decode_byte(byte2);

            let mut all_pens = [0u8; 16];
            let len1 = pens1.len();
            all_pens[..len1].copy_from_slice(pens1);
            all_pens[len1..(len1 + pens2.len())].copy_from_slice(pens2);

            self.framebuffer.push_16_pixels(
                self.gate_array.video_mode,
                &all_pens,
                &self.gate_array.pens_rgb,
            );
        } else {
            // Hors de la zone d'affichage, le Gate Array émet la couleur de bordure
            self.framebuffer
                .push_16_border_pixels(self.gate_array.border_rgb);
        }
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
