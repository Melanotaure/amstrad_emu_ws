// AY-3-8912 Logarithmic conversion table (-3 dB per step)
const AY_VOLUME_TABLE: [f32; 16] = [
    0.000000, // 0 : Silence
    0.007812, // 1
    0.011048, // 2
    0.015625, // 3
    0.022097, // 4
    0.031250, // 5
    0.044194, // 6
    0.062500, // 7
    0.088388, // 8
    0.125000, // 9
    0.176776, // 10
    0.250000, // 11
    0.353553, // 12
    0.500000, // 13 : Half of the max amplitude (-6 dB)
    0.707106, // 14 : 1 / sqrt(2) (-3 dB)
    1.000000, // 15 : Max Amplitude (0 dB)
];

pub struct PsgAy {
    pub registers: [u8; 16],
    pub selected_register: usize,

    // Keyboard matrix: 10 rows of 8 keys (0xFF = no key pressed)
    pub keyboard_matrix: [u8; 10],

    phase_a: f32,
    phase_b: f32,
    phase_c: f32,

    noise_lfsr: u32,
    noise_phase: f32,
    noise_level: bool,

    env_phase: f32,
    env_step: i8,
    env_direction: i8,
    env_holding: bool,
    env_volume: usize,

    sample_rate: f32,
}

impl PsgAy {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            registers: [0; 16],
            selected_register: 0,
            keyboard_matrix: [0xFF; 10], // All the keys are released at start
            phase_a: 0.0,
            phase_b: 0.0,
            phase_c: 0.0,
            noise_lfsr: 1,
            noise_phase: 0.0,
            noise_level: true,
            env_phase: 0.0,
            env_step: 0,
            env_direction: 1,
            env_holding: false,
            env_volume: 0,
            sample_rate,
        }
    }

    /// Executes one command of the PSG bus and returns a potential byte to be injected in the PPI port A
    pub fn update_bus(
        &mut self,
        bdir: bool,
        bc1: bool,
        ppi_port_a_out: u8,
        selected_line: usize,
    ) -> Option<u8> {
        match (bdir, bc1) {
            // Inactive
            (false, false) => None,
            // Address Latch (%11): active register selection
            (true, true) => {
                self.selected_register = (ppi_port_a_out & 0x0F) as usize;
                None
            }
            // Data write (%10): writes into the selected register
            (true, false) => {
                if self.selected_register < 16 {
                    self.registers[self.selected_register] = ppi_port_a_out;
                }
                None
            }
            // Data read (%01): reads the selected register
            (false, true) => {
                if self.selected_register == 14 {
                    // Register 14 = PSG I/O Port -> reading of the keyboard matrix
                    if selected_line < 10 {
                        Some(self.keyboard_matrix[selected_line])
                    } else {
                        Some(0xFF)
                    }
                } else if self.selected_register < 16 {
                    Some(self.registers[self.selected_register])
                } else {
                    Some(0xFF)
                }
            }
        }
    }

    /// Modify the state of one key (called by the OS window events)
    pub fn set_key_state(&mut self, line: usize, bit: usize, pressed: bool) {
        if line < 10 && bit < 8 {
            if pressed {
                self.keyboard_matrix[line] &= !(1 << bit); // 0 = pressed
            } else {
                self.keyboard_matrix[line] |= 1 << bit; // 1 = released
            }
        }
    }

    pub fn write_data(&mut self, data: u8) {
        if self.selected_register < 16 {
            self.registers[self.selected_register] = data;

            if self.selected_register == 13 {
                self.reset_envelope();
            }
        }
    }

    pub fn reset_envelope(&mut self) {
        let shape = self.registers[13] & 0x0F;
        let attack = (shape & 0x04) != 0;

        self.env_phase = 0.0;
        self.env_holding = false;

        if attack {
            self.env_step = 0;
            self.env_direction = 1;
        } else {
            self.env_step = 15;
            self.env_direction = -1;
        }
        self.env_volume = self.env_step as usize;
    }

    /// Called once per audio sample (ex: 48000 times per second)
    pub fn generate_sample(&mut self) -> f32 {
        let ep = ((self.registers[12] as u16) << 8) | (self.registers[11] as u16);
        let ep = if ep == 0 { 1 } else { ep };

        let env_freq = 1_000_000.0 / (256.0 * ep as f32);
        self.env_phase += env_freq / self.sample_rate;

        while self.env_phase >= 1.0 {
            self.env_phase -= 1.0;
            self.step_envelope();
        }

        // Noise generator update
        let np = self.registers[6] & 0x1F;
        let noise_period = if np == 0 { 1 } else { np };

        let noise_freq = 1_000_000.0 / (16.0 * noise_period as f32);
        let noise_inc = noise_freq / self.sample_rate;

        self.noise_phase += noise_inc;

        while self.noise_phase >= 1.0 {
            self.noise_phase -= 1.0;

            let bit0 = self.noise_lfsr & 1;
            let bit3 = (self.noise_lfsr >> 3) & 1;
            let feedback = bit0 ^ bit3;

            self.noise_lfsr = (self.noise_lfsr >> 1) | (feedback << 16);

            self.noise_level = bit0 == 1;
        }

        let mixer = self.registers[7];
        let tone_a_enabled = (mixer & 0x01) == 0;
        let tone_b_enabled = (mixer & 0x02) == 0;
        let tone_c_enabled = (mixer & 0x04) == 0;
        let noise_a_enabled = (mixer & 0x08) == 0;
        let noise_b_enabled = (mixer & 0x10) == 0;
        let noise_c_enabled = (mixer & 0x20) == 0;

        let sample_a = self.mix_channel(0, 1, 8, 0, tone_a_enabled, noise_a_enabled);
        let sample_b = self.mix_channel(2, 3, 9, 1, tone_b_enabled, noise_b_enabled);
        let sample_c = self.mix_channel(4, 5, 10, 2, tone_c_enabled, noise_c_enabled);

        (sample_a + sample_b + sample_c) / 3.0
    }

    fn mix_channel(
        &mut self,
        reg_fine: usize,
        reg_coarse: usize,
        reg_vol: usize,
        phase_nb: u32,
        tone_en: bool,
        noise_en: bool,
    ) -> f32 {
        let tp =
            ((self.registers[reg_coarse] as u16 & 0x0F) << 8) | (self.registers[reg_fine] as u16);

        let tone_level = if tp == 0 {
            true
        } else {
            let freq = 1_000_000.0 / (16.0 * tp as f32);

            if phase_nb == 0 {
                self.phase_a += freq / self.sample_rate;
                if self.phase_a > 1.0 {
                    self.phase_a -= 1.0;
                }
                self.phase_a < 0.5
            } else if phase_nb == 1 {
                self.phase_b += freq / self.sample_rate;
                if self.phase_b > 1.0 {
                    self.phase_b -= 1.0;
                }
                self.phase_b < 0.5
            } else {
                self.phase_c += freq / self.sample_rate;
                if self.phase_c > 1.0 {
                    self.phase_c -= 1.0;
                }
                self.phase_c < 0.5
            }
        };

        let t_val = tone_level || !tone_en;
        let n_val = self.noise_level || !noise_en;

        let signal_active = t_val && n_val;

        let use_envelope = (self.registers[reg_vol] & 0x10) != 0;
        let vol_index = if use_envelope {
            self.env_volume
        } else {
            (self.registers[reg_vol] & 0x0F) as usize
        };
        let volume = AY_VOLUME_TABLE[vol_index];

        if signal_active {
            volume
        } else {
            -volume
        }
    }

    fn step_envelope(&mut self) {
        if self.env_holding {
            return; // The envelop is held until the next writing in R13
        }

        self.env_step += self.env_direction;

        // Overflow: end of one cycle (over 15, or under 0)
        if self.env_step < 0 || self.env_step > 15 {
            let shape = self.registers[13] & 0x0F;
            let cont = (shape & 0x08) != 0; // Bit 3 : Continue
            let attack = (shape & 0x04) != 0; // Bit 2 : Attack
            let alt = (shape & 0x02) != 0; // Bit 1 : Alternate
            let hold = (shape & 0x01) != 0; // Bit 0 : Hold

            if !cont {
                // Forms 0 to 7 : One cycle, then held to 0 (\___)
                self.env_holding = true;
                self.env_volume = 0;
            } else {
                // Forms 8 to 15 (Continue)
                if alt {
                    self.env_direction = -self.env_direction; // Bounce
                }
                if hold {
                    self.env_holding = true;
                }

                self.env_step = if self.env_direction > 0 { 0 } else { 15 };
                self.env_volume = self.env_step as usize;

                if self.env_holding {
                    // Held volume is determined by an exclusive OR between Attack and Alternate
                    self.env_volume = if attack ^ alt { 15 } else { 0 };
                }
            }
        } else {
            self.env_volume = self.env_step as usize;
        }
    }
}
