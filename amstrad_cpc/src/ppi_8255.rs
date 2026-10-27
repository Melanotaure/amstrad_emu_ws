pub struct Ppi8255 {
    pub port_a_out: u8,
    pub port_a_in: u8,
    pub port_c: u8,

    // PPI8255 control registers
    pub port_a_input: bool,
    pub port_b_input: bool,
    pub port_c_lower_input: bool,
    pub port_c_upper_input: bool,
}

impl Ppi8255 {
    pub fn new() -> Self {
        Self {
            port_a_out: 0,
            port_a_in: 0xFF,
            port_c: 0,
            port_a_input: false,
            port_b_input: true,
            port_c_lower_input: false,
            port_c_upper_input: false,
        }
    }

    pub fn read_io(&self, port: u16, vsync: bool) -> u8 {
        let function = ((port >> 8) & 0x03) as u8;

        match function {
            0x00 => {
                // Port A (0xF4xx): data PSG/Keyboard
                if self.port_a_input {
                    self.port_a_in
                } else {
                    self.port_a_out
                }
            }
            0x01 => {
                // Port B (0xF5xx): System input & VSYNC
                // 0x7E = 0b01111110 (bits 1..6 set for UK/FR 50Hz Amstrad)
                let base = 0x7E;
                if vsync {
                    // println!("[PPI] Read VSYNC ON");
                    base | 0x01
                } else {
                    base
                }
            }
            0x02 => {
                // Port C (0xF6xx): returns the current state of port C
                self.port_c
            }
            _ => 0xFF,
        }
    }

    pub fn write_io(&mut self, port: u16, data: u8) {
        let function = ((port >> 8) & 0x03) as u8;

        match function {
            0x00 => {
                // Port A (0xF4xx)
                self.port_a_out = data;
            }
            0x01 => {
                // Port B (0xF5xx): Generally configured as input, writing has no effect.
            }
            0x02 => {
                // Port C (0xF6xx): Keyboard line, cassette, PSG control
                self.port_c = data;
            }
            0x03 => {
                // Control register (0xF7xx)
                if (data & 0x80) != 0 {
                    // Port configuration Mode
                    self.port_a_input = (data & 0x10) != 0;
                    self.port_c_upper_input = (data & 0x08) != 0;
                    self.port_b_input = (data & 0x02) != 0;
                    self.port_c_lower_input = (data & 0x01) != 0;
                } else {
                    // Port C bit set/reset Mode
                    let bit = (data >> 1) & 0x07;
                    let set = (data & 0x01) != 0;
                    if set {
                        self.port_c |= 1 << bit;
                    } else {
                        self.port_c &= !(1 << bit);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
