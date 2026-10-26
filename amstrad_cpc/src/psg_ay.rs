pub struct PsgAy {
    pub registers: [u8; 16],
    pub selected_register: usize,

    // Keyboard matrix: 10 rows of 8 keys (0xFF = no key pressed)
    pub keyboard_matrix: [u8; 10],
}

impl PsgAy {
    pub fn new() -> Self {
        Self {
            registers: [0; 16],
            selected_register: 0,
            keyboard_matrix: [0xFF; 10], // All the keys are released at start
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
}
