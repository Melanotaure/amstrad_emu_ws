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

    // 2. VLC (Vertical Line Counter): counts the raster lines from 0 to R9 (also called RA)
    pub vlc: u8,

    // 3. VCC (Vertical Character Counter): counts the character rows from 0 to R4
    pub vcc: u8,

    // --- Generated address pointers ---
    pub ma: u16,           // current address (incremented at each cycle)
    pub ma_row_start: u16, // Memorize the start address of the current row

    pub vsync_countdown: u8,
}

impl Crtc {
    pub fn new() -> Self {
        Self {
            registers: [0; 18],
            selected_register: 0,
            hcc: 0,
            vlc: 0,
            vcc: 0,
            ma: 0,
            ma_row_start: 0,
            vsync_countdown: 0,
        }
    }

    pub fn clock(&mut self) -> CrtcOutput {
        let r0 = self.registers[0];
        let r2 = self.registers[2];
        let hsw = self.get_hsw();

        // 1. État des signaux pour le cycle actuel
        let display_enable = self.hcc < self.registers[1] && self.vcc < self.registers[6];

        let hsync = if r0 > 0 {
            if r2 + hsw <= r0 {
                self.hcc >= r2 && self.hcc < r2 + hsw
            } else {
                self.hcc >= r2 || self.hcc < (r2 + hsw) % (r0 + 1)
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

        // 2. Avancement des compteurs pour le cycle suivant
        self.ma = (self.ma + 1) & 0x3FFF;
        self.hcc += 1;

        // Fin de la ligne horizontale (R0)
        if self.hcc > r0 {
            self.hcc = 0;
            self.vlc += 1;

            // Si un VSYNC est en cours, il dure 1 scanline de moins
            if self.vsync_countdown > 0 {
                self.vsync_countdown -= 1;
            }

            // Fin de la rangée de caractères (R9)
            if self.vlc > self.registers[9] {
                self.vlc = 0;
                self.vcc += 1;

                // La rangée suivante démarre à +R1 (40), pas à +R0+1 (64)
                self.ma_row_start = (self.ma_row_start + self.registers[1] as u16) & 0x3FFF;

                // Fin de la trame complète (R4)
                if self.vcc > self.registers[4] {
                    self.vcc = 0;
                    let start_addr =
                        ((self.registers[12] as u16) << 8) | (self.registers[13] as u16);
                    self.ma_row_start = start_addr & 0x3FFF;
                }

                // Déclenchement du VSYNC quand on atteint la rangée R7
                if self.vcc == self.registers[7] {
                    self.vsync_countdown = self.get_vsw();
                }
            }

            // Retour chariot mémoire au début de la rangée courante
            self.ma = self.ma_row_start;
        }

        output
    }

    /// Called when an OUT (C), r by the Z80
    pub fn write_io(&mut self, port: u16, value: u8) {
        // Port decoding on the Amstrad
        let port_function = (port >> 8) & 0x03;

        match port_function {
            0x00 => {
                // Port 0xBCxx : Register selection (only 5 bits are useful, so max val 31)
                self.selected_register = (value & 0x1F) as usize;
                // println!("[CRTC] Sélection du Registre R{}", self.selected_register);
            }
            0x01 => {
                // Port 0xBDxx : Data writing
                if self.selected_register < 18 {
                    self.registers[self.selected_register] = value;
                    // println!("[CRTC] Écriture Donnée = 0x{:02X}", value);
                }
            }
            _ => {
                // Ignore the other ports (like 0xBFxx that is used for reading on some CRTC)
            }
        }
    }

    /// Récupère la largeur HSYNC en cycles de caractères (1 à 16)
    pub fn get_hsw(&self) -> u8 {
        let hsw = (self.registers[3] >> 4) & 0x0F;
        if hsw == 0 {
            16
        } else {
            hsw
        }
    }

    /// Récupère la largeur VSYNC en lignes (1 à 16)
    pub fn get_vsw(&self) -> u8 {
        let vsw = self.registers[3] & 0x0F;
        if vsw == 0 {
            16
        } else {
            vsw
        }
    }
}
