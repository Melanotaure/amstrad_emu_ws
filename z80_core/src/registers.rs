use crate::flags::Flags;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub ixh: u8,
    pub ixl: u8,
    pub iyh: u8,
    pub iyl: u8,
    pub i: u8,
    pub r: u8,
    pub sp: u16,
    pub pc: u16,
    pub flags: Flags,
    // Extra regs
    pub eaf: u16,
    pub ebc: u16,
    pub ede: u16,
    pub ehl: u16,
    pub wz: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0xFF,
            b: 0xFF,
            c: 0xFF,
            d: 0xFF,
            e: 0xFF,
            h: 0xFF,
            l: 0xFF,
            ixh: 0xFF,
            ixl: 0xFF,
            iyh: 0xFF,
            iyl: 0xFF,
            i: 0xFF,
            r: 0x00,
            sp: 0xFFFF,
            pc: 0x0000,
            flags: Flags::new(),
            eaf: 0xFFFF,
            ebc: 0xFFFF,
            ede: 0xFFFF,
            ehl: 0xFFFF,
            wz: 0xFFFF,
        }
    }

    pub fn get_af(&self) -> u16 {
        u16::from_le_bytes([self.flags.to_byte(), self.a])
    }

    pub fn set_af(&mut self, val: u16) {
        let f: u8;
        [f, self.a] = val.to_le_bytes();
        self.flags.from_byte(f);
    }

    pub fn get_bc(&self) -> u16 {
        u16::from_le_bytes([self.c, self.b])
    }

    pub fn set_bc(&mut self, val: u16) {
        [self.c, self.b] = val.to_le_bytes();
    }

    pub fn get_de(&self) -> u16 {
        u16::from_le_bytes([self.e, self.d])
    }

    pub fn set_de(&mut self, val: u16) {
        [self.e, self.d] = val.to_le_bytes();
    }

    pub fn get_hl(&self) -> u16 {
        u16::from_le_bytes([self.l, self.h])
    }

    pub fn set_hl(&mut self, val: u16) {
        [self.l, self.h] = val.to_le_bytes();
    }

    pub fn get_ix(&self) -> u16 {
        u16::from_le_bytes([self.ixl, self.ixh])
    }

    pub fn set_ix(&mut self, val: u16) {
        [self.ixl, self.ixh] = val.to_le_bytes();
    }

    pub fn get_iy(&self) -> u16 {
        u16::from_le_bytes([self.iyl, self.iyh])
    }

    pub fn set_iy(&mut self, val: u16) {
        [self.iyl, self.iyh] = val.to_le_bytes();
    }

    pub fn get_ir(&self) -> u16 {
        u16::from_le_bytes([self.r, self.i])
    }

    pub fn set_ir(&mut self, val: u16) {
        [self.r, self.i] = val.to_le_bytes();
    }

    pub fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn dec_pc(&mut self) {
        self.pc = self.pc.wrapping_sub(1);
    }

    pub fn inc_sp(&mut self) {
        self.sp = self.sp.wrapping_add(1);
    }

    pub fn dec_sp(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn inc_r(&mut self) {
        self.r = self.r.wrapping_add(1);
    }

    pub fn reset(&mut self) {
        self.a = 0xFF;
        self.b = 0xFF;
        self.c = 0xFF;
        self.d = 0xFF;
        self.e = 0xFF;
        self.h = 0xFF;
        self.l = 0xFF;
        self.ixh = 0xFF;
        self.ixl = 0xFF;
        self.iyh = 0xFF;
        self.iyl = 0xFF;
        self.i = 0xFF;
        self.r = 0x00;
        self.sp = 0xFFFF;
        self.pc = 0x0000;
        self.flags.reset();
    }
}
