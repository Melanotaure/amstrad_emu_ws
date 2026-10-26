pub const SCREEN_WIDTH: usize = 768;
pub const SCREEN_HEIGHT: usize = 288;

const HBLANK_CYCLES: usize = 14;
const VBLANK_LINES: usize = 28;
const VISIBLE_CYCLES: usize = SCREEN_WIDTH / 16;

pub struct Framebuffer {
    pub pixels: Vec<u32>,

    pub h_cycle: usize,
    pub v_line: usize,

    prev_hsync: bool,
    prev_vsync: bool,
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            h_cycle: 0,
            v_line: 0,
            prev_hsync: false,
            prev_vsync: false,
        }
    }

    /// Synchronize the beam with the CRTC signals at the beginning of each cycle (1 µs)
    pub fn sync_beam(&mut self, hsync: bool, vsync: bool) {
        // 1. HSYNC rising edge: the starts back at the very left of the tube
        if hsync && !self.prev_hsync {
            self.h_cycle = 0;
            self.v_line += 1;
        }

        // 2. VSYNC rising edge : the starts back at the very top of the tube
        if vsync && !self.prev_vsync {
            self.v_line = 0;
        }

        self.prev_hsync = hsync;
        self.prev_vsync = vsync;
    }

    pub fn push_16_pixels(&mut self, mode: u8, pens: &[u8], palette: &[u32]) {
        if let Some(base_index) = self.current_buffer_index() {
            match mode {
                0 => {
                    for (i, &pen) in pens[0..4].iter().enumerate() {
                        let rgb = palette[pen as usize];
                        for j in 0..4 {
                            self.pixels[base_index + (i * 4) + j] = rgb;
                        }
                    }
                }
                1 => {
                    for (i, &pen) in pens[0..8].iter().enumerate() {
                        let rgb = palette[pen as usize];
                        for j in 0..2 {
                            self.pixels[base_index + (i * 2) + j] = rgb;
                        }
                    }
                }
                _ => {
                    for (i, &pen) in pens[0..16].iter().enumerate() {
                        self.pixels[base_index + i] = palette[pen as usize];
                    }
                }
            }
        }

        // Le faisceau avance d'un cycle (16 micro-pixels)
        self.h_cycle += 1;
    }

    /// Dessine 16 pixels de bordure (quand display_enable est faux)
    pub fn push_16_border_pixels(&mut self, border_rgb: u32) {
        if let Some(base_index) = self.current_buffer_index() {
            self.pixels[base_index..base_index + 16].fill(border_rgb);
        }

        self.h_cycle += 1;
    }

    /// Calcule l'index dans le tableau `pixels` si le faisceau est dans la zone visible
    #[inline(always)]
    fn current_buffer_index(&self) -> Option<usize> {
        if self.h_cycle >= HBLANK_CYCLES
            && self.h_cycle < (HBLANK_CYCLES + VISIBLE_CYCLES)
            && self.v_line >= VBLANK_LINES
            && self.v_line < (VBLANK_LINES + SCREEN_HEIGHT)
        {
            let beam_x = (self.h_cycle - HBLANK_CYCLES) * 16;
            let beam_y = self.v_line - VBLANK_LINES;
            Some(beam_y * SCREEN_WIDTH + beam_x)
        } else {
            None
        }
    }
}
