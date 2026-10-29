pub struct CrtcOutput {
    pub ma: u16,
    pub ra: u8,
    pub display_enable: bool,
    pub hsync: bool,
    pub vsync: bool,
}

pub struct Crtc {
    pub registers: [u8; 18],
    pub selected_register: usize,

    // --- Internal counters (Scanning state) ---

    // 1. HCC (Horizontal Character Counter): counts colums from 0 to R0
    pub hcc: u8,

    // True while the R5 vertical total adjust lines are scanned
    pub in_adjust: bool,

    // 2. VLC (Vertical Line Counter): counts the raster lines from 0 to R9 (also called RA)
    pub vlc: u8,

    // 3. VCC (Vertical Character Counter): counts the character rows from 0 to R4
    pub vcc: u8,

    // --- Generated address pointers ---
    pub ma: u16,           // current address (incremented at each cycle)
    pub ma_row_start: u16, // Memorize the start address of the current row

    pub vsync_countdown: u8,
}

impl Default for Crtc {
    fn default() -> Self {
        Self::new()
    }
}

impl Crtc {
    pub fn new() -> Self {
        Self {
            registers: [0; 18],
            selected_register: 0,
            hcc: 0,
            in_adjust: false,
            vlc: 0,
            vcc: 0,
            ma: 0,
            ma_row_start: 0,
            vsync_countdown: 0,
        }
    }

    pub fn clock(&mut self) -> CrtcOutput {
        // u16 arithmetic: R0 / R2 / R9 can be 255 and must not overflow
        let r0 = self.registers[0] as u16;
        let r2 = self.registers[2] as u16;
        let hsw = self.get_hsw() as u16;
        let hcc = self.hcc as u16;

        // 1. Current cycle signal state
        let display_enable = !self.in_adjust
            && self.hcc < self.registers[1]
            && self.vcc < self.registers[6];

        let hsync = if r0 > 0 {
            if r2 + hsw <= r0 + 1 {
                hcc >= r2 && hcc < r2 + hsw
            } else {
                // The pulse wraps around the end of the line
                hcc >= r2 || hcc < (r2 + hsw) % (r0 + 1)
            }
        } else {
            false
        };

        let vsync = self.vsync_countdown > 0;

        let output = CrtcOutput {
            ma: self.ma,
            ra: self.vlc,
            display_enable,
            hsync,
            vsync,
        };

        // 2. Incrementing the counters for the next cycle
        self.ma = (self.ma + 1) & 0x3FFF;

        // Horizontal line end (R0)
        if hcc < r0 {
            self.hcc += 1;
            return output;
        }
        self.hcc = 0;

        // If a VSYNC is ongoing, it lasts 1 scanline less
        if self.vsync_countdown > 0 {
            self.vsync_countdown -= 1;
        }

        if self.in_adjust {
            // R5: extra raster lines after the last character row
            self.vlc += 1;
            if self.vlc >= self.registers[5] {
                self.start_frame();
            }
        } else if self.vlc < self.registers[9] {
            self.vlc += 1;
        } else {
            // End of the characters row (R9)
            self.vlc = 0;

            if self.vcc < self.registers[4] {
                self.vcc += 1;
                // The next row starts at +R1 (40), not at +R0+1 (64)
                self.ma_row_start = (self.ma_row_start + self.registers[1] as u16) & 0x3FFF;
            } else if self.registers[5] > 0 {
                // End of the raster (R4): vertical total adjust (R5)
                self.in_adjust = true;
            } else {
                self.start_frame();
            }

            // VSYNC triggers when row R7 is reached
            if !self.in_adjust && self.vcc == self.registers[7] {
                self.vsync_countdown = self.get_vsw();
            }
        }

        // Memory carriage return at the start of the current row
        self.ma = self.ma_row_start;

        output
    }

    /// Restart the scanning at the top of the frame
    fn start_frame(&mut self) {
        self.in_adjust = false;
        self.vlc = 0;
        self.vcc = 0;
        let start_addr = ((self.registers[12] as u16) << 8) | (self.registers[13] as u16);
        self.ma_row_start = start_addr & 0x3FFF;
    }

    /// Called when an OUT (C), r by the Z80
    pub fn write_io(&mut self, port: u16, value: u8) {
        // Port decoding on the Amstrad
        let port_function = (port >> 8) & 0x03;

        match port_function {
            0x00 => {
                // Port 0xBCxx : Register selection (only 5 bits are useful, so max val 31)
                self.selected_register = (value & 0x1F) as usize;
                // println!("[CRTC] Selected Register R{}", self.selected_register);
            }
            0x01 => {
                // Port 0xBDxx : Data writing
                if self.selected_register < 18 {
                    self.registers[self.selected_register] = value;
                    // println!("[CRTC] Data writing = 0x{:02X}", value);
                }
            }
            _ => {
                // Ignore the other ports (like 0xBFxx that is used for reading on some CRTC)
            }
        }
    }

    /// Get the width of HSYNC in character cycles (1 to 16)
    pub fn get_hsw(&self) -> u8 {
        let hsw = (self.registers[3] >> 4) & 0x0F;
        if hsw == 0 {
            16
        } else {
            hsw
        }
    }

    /// Get the width of VSYNC in lines (1 to 16)
    pub fn get_vsw(&self) -> u8 {
        let vsw = self.registers[3] & 0x0F;
        if vsw == 0 {
            16
        } else {
            vsw
        }
    }
}
