pub const SCREEN_WIDTH: usize = 768;
pub const SCREEN_HEIGHT: usize = 288;

pub struct Framebuffer {
    pub pixels: Vec<u32>,

    pub beam_x: usize,
    pub beam_y: usize,
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            beam_x: 0,
            beam_y: 0,
        }
    }

    pub fn reset_beam(&mut self) {
        self.beam_x = 0;
        self.beam_y = 0;
    }

    pub fn push_16_pixels(&mut self, mode: u8, pens: &[u8], palette: &[u32]) {
        if self.beam_x >= SCREEN_WIDTH || self.beam_y >= SCREEN_HEIGHT {
            self.beam_x += 16;
            return;
        }

        let base_index = self.beam_y * SCREEN_WIDTH + self.beam_x;

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
            2 | _ => {
                for (i, &pen) in pens[0..16].iter().enumerate() {
                    self.pixels[base_index + i] = palette[pen as usize];
                }
            }
        }

        self.beam_x += 16;
    }
}
