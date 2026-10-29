pub struct GateArray {
    pub video_mode: u8,

    selected_pen: u8,
    pub pens_rgb: [u32; 16],
    pub border_rgb: u32,

    // Interrupt management
    pub line_counter: u8,
    pub vsync_delay: u8,
    pub interrupt_pending: bool,

    prev_hsync: bool,
    prev_vsync: bool,

    // Video LUTs
    lut_mode0: [[u8; 2]; 256],
    lut_mode1: [[u8; 4]; 256],
    lut_mode2: [[u8; 8]; 256],
    lut_mode3: [[u8; 2]; 256],
}

impl Default for GateArray {
    fn default() -> Self {
        Self::new()
    }
}

impl GateArray {
    pub fn new() -> Self {
        Self {
            video_mode: 1, // Default starting mode
            selected_pen: 0,
            pens_rgb: [0; 16],
            border_rgb: 0,
            line_counter: 0,
            vsync_delay: 0,
            interrupt_pending: false,
            prev_hsync: false,
            prev_vsync: false,
            lut_mode0: Self::build_lut_mode0(),
            lut_mode1: Self::build_lut_mode1(),
            lut_mode2: Self::build_lut_mode2(),
            lut_mode3: Self::build_lut_mode3(),
        }
    }

    /// Called at each CRTC cycle (1 µs) with the current sync signals
    pub fn update_sync(&mut self, hsync: bool, vsync: bool) {
        // VSYNC RISING EDGE: the synchronisation starts 2 HSYNCs later
        if vsync && !self.prev_vsync {
            self.vsync_delay = 2;
        }

        // HSYNC FALLING EDGE: Normal line count
        if !hsync && self.prev_hsync {
            self.line_counter += 1;
            if self.line_counter >= 52 {
                self.line_counter = 0;
                self.interrupt_pending = true;
            }

            if self.vsync_delay > 0 {
                self.vsync_delay -= 1;
                if self.vsync_delay == 0 {
                    // If the counter is >= 32, the interrupt is raised; in any case it restarts from 0
                    if self.line_counter >= 32 {
                        self.interrupt_pending = true;
                    }
                    self.line_counter = 0;
                }
            }
        }

        self.prev_hsync = hsync;
        self.prev_vsync = vsync;
    }

    /// Called when Z80 sends the command %10xxxxxx with bit 4 set
    pub fn clear_interrupt_counter(&mut self) {
        self.line_counter = 0;
        self.interrupt_pending = false;
    }

    /// Called by the Z80 exactly when it acknowledge the interrupt (INTA)
    pub fn acknowledge_interrupt(&mut self) {
        self.interrupt_pending = false;
        self.line_counter &= 0x1F; // resets bit 5
    }

    /// Called when the Z80 send %10xxxxxxx
    pub fn set_video_mode(&mut self, data: u8) {
        self.video_mode = data;
    }

    /// Called when the Z80 sends %000xxxxx
    pub fn select_pen(&mut self, data: u8) {
        // Bit 4 selects the border (whatever the lower bits), otherwise the pen number is 0..15
        self.selected_pen = if (data & 0x10) != 0 { 16 } else { data & 0x0F };
    }

    /// Called when the Z80 sends %010xxxxx
    pub fn set_color(&mut self, data: u8) {
        let hardware_color_index = (data & 0x1F) as usize;
        let rgb_value = HARDWARE_COLORS_RGB[hardware_color_index];

        if self.selected_pen < 16 {
            self.pens_rgb[self.selected_pen as usize] = rgb_value;
        } else {
            self.border_rgb = rgb_value;
        }
    }

    fn build_lut_mode0() -> [[u8; 2]; 256] {
        let mut lut = [[0; 2]; 256];

        for b in 0..=255 {
            let p0 = ((b >> 7) & 1)
                | (((b >> 3) & 1) << 1)
                | (((b >> 5) & 1) << 2)
                | (((b >> 1) & 1) << 3);
            let p1 =
                ((b >> 6) & 1) | (((b >> 2) & 1) << 1) | (((b >> 4) & 1) << 2) | ((b & 1) << 3);

            lut[b as usize] = [p0, p1];
        }
        lut
    }

    /// Mode 3 (undocumented): 2 pixels per byte, 2 bits per pixel (bits 5, 4, 1 and 0 are unused)
    fn build_lut_mode3() -> [[u8; 2]; 256] {
        let mut lut = [[0; 2]; 256];
        for b in 0..=255u8 {
            let p0 = ((b >> 7) & 1) | (((b >> 3) & 1) << 1);
            let p1 = ((b >> 6) & 1) | (((b >> 2) & 1) << 1);
            lut[b as usize] = [p0, p1];
        }
        lut
    }

    fn build_lut_mode1() -> [[u8; 4]; 256] {
        let mut lut = [[0; 4]; 256];
        for b in 0..=255 {
            let p0 = ((b >> 7) & 1) | (((b >> 3) & 1) << 1);
            let p1 = ((b >> 6) & 1) | (((b >> 2) & 1) << 1);
            let p2 = ((b >> 5) & 1) | (((b >> 1) & 1) << 1);
            let p3 = ((b >> 4) & 1) | ((b & 1) << 1);
            lut[b as usize] = [p0, p1, p2, p3];
        }
        lut
    }

    fn build_lut_mode2() -> [[u8; 8]; 256] {
        let mut lut = [[0; 8]; 256];
        for b in 0..=255 {
            for i in 0..8 {
                // Pixel 0 takes bit 7, Pixel 7 takes bit 0
                lut[b as usize][i] = (b >> (7 - i)) & 1;
            }
        }
        lut
    }

    // --- Real time decoding ---

    pub fn decode_byte(&self, byte: u8) -> &[u8] {
        match self.video_mode {
            0 => &self.lut_mode0[byte as usize],
            1 => &self.lut_mode1[byte as usize],
            2 => &self.lut_mode2[byte as usize],
            _ => &self.lut_mode3[byte as usize],
        }
    }
}

/// Conversion table from hardware color of the Gate Array (0 -> 31) to XRGB format (0x00RRGGBB)
const HARDWARE_COLORS_RGB: [u32; 32] = [
    0x007F7F7F, // 00 : White (50%)
    0x007F7F7F, // 01 : White (50%) - Doublon
    0x0000FF7F, // 02 : Sea Green
    0x00FFFF7F, // 03 : Pastel Yellow
    0x0000007F, // 04 : Blue
    0x00FF007F, // 05 : Purple
    0x00007F7F, // 06 : Cyan
    0x00FF7F7F, // 07 : Pink
    0x00FF007F, // 08 : Purple - Doublon
    0x00FFFF7F, // 09 : Pastel Yellow - Doublon
    0x00FFFF00, // 10 : Bright Yellow
    0x00FFFFFF, // 11 : Bright White
    0x00FF0000, // 12 : Bright Red
    0x00FF00FF, // 13 : Bright Magenta
    0x00FF7F00, // 14 : Orange
    0x00FF7FFF, // 15 : Pastel Magenta
    0x0000007F, // 16 : Blue - Doublon
    0x0000FF7F, // 17 : Sea Green - Doublon
    0x0000FF00, // 18 : Bright Green
    0x0000FFFF, // 19 : Bright Cyan
    0x00000000, // 20 : Black
    0x000000FF, // 21 : Bright Blue
    0x00007F00, // 22 : Green
    0x00007FFF, // 23 : Sky Blue
    0x007F007F, // 24 : Magenta
    0x007FFF7F, // 25 : Pastel Green
    0x007FFF00, // 26 : Lime
    0x007FFFFF, // 27 : Pastel Cyan
    0x007F0000, // 28 : Red
    0x007F00FF, // 29 : Mauve
    0x007F7F00, // 30 : Yellow
    0x007F7FFF, // 31 : Pastel Blue
];
