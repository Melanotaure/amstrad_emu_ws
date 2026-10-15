// Special management for flags
pub struct Flags {
    pub s: bool,   // sign                 : bit 7
    pub z: bool,   // zero                 : bit 6
    pub b5: bool,  // unused               : bit 5
    pub h: bool,   // half carry           : bit 4
    pub b3: bool,  // unused               : bit 3
    pub p: bool,   // parity / overflow    : bit 2
    pub n: bool,   // subtract             : bit 1
    pub c: bool,   // carry                : bit 0
    pub alu: bool, // alu computed flag    : xxx x
}

impl Flags {
    pub fn new() -> Self {
        Self {
            s: true,
            z: true,
            b5: true,
            h: true,
            b3: true,
            p: true,
            n: true,
            c: true,
            alu: false,
        }
    }

    pub fn to_byte(&self) -> u8 {
        [
            self.s, self.z, self.b5, self.h, self.b3, self.p, self.n, self.c,
        ]
        .iter()
        .rev()
        .enumerate()
        .fold(0, |acc, (i, b)| acc | (*b as u8) << i)
    }

    pub fn from_byte(&mut self, val: u8) {
        self.s = (val & 0x80) != 0;
        self.z = (val & 0x40) != 0;
        self.b5 = (val & 0x20) != 0;
        self.h = (val & 0x10) != 0;
        self.b3 = (val & 0x08) != 0;
        self.p = (val & 0x04) != 0;
        self.n = (val & 0x02) != 0;
        self.c = (val & 0x01) != 0;
    }

    pub fn reset(&mut self) {
        self.s = false;
        self.z = false;
        self.b5 = false;
        self.h = false;
        self.b3 = false;
        self.p = false;
        self.n = false;
        self.c = false;
        self.alu = false;
    }
}
