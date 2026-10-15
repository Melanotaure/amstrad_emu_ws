use crate::bus::SystemBus;
use crate::{cycles::CYCLES_ED, z80::*};

impl Z80 {
    fn in_r_c<B: SystemBus>(&mut self, bus: &mut B) -> u8 {
        let addr = self.reg.get_bc();
        let data = bus.read_io(addr);
        self.reg.flags.s = data & 0x80 == 0x80;
        self.reg.flags.z = data == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = data.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        data
    }

    fn out_c_r<B: SystemBus>(&mut self, reg: u8, bus: &mut B) {
        let addr = self.reg.get_bc();
        bus.write_io(addr, reg);
        self.reg.flags.alu = false;
    }

    fn sbc_hl_rr(&mut self, reg: u16) -> u16 {
        let c = self.reg.flags.c as u16;
        let hl = self.reg.get_hl();
        let r = hl.wrapping_sub(reg.wrapping_add(c));
        self.reg.flags.s = r & 0x8000 == 0x8000;
        self.reg.flags.z = r == 0x0000;
        self.reg.flags.h = (hl ^ reg ^ r) & 0x1000 != 0;
        self.reg.flags.p = ((hl ^ reg) & (hl ^ r) & 0x8000) != 0;
        self.reg.flags.n = true;
        self.reg.flags.c = (hl as u32) < (reg as u32 + c as u32);
        self.reg.flags.b5 = r & 0b00100000_00000000 != 0;
        self.reg.flags.b3 = r & 0b00001000_00000000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        r
    }

    fn adc_hl_rr(&mut self, reg: u16) -> u16 {
        let c = self.reg.flags.c as u16;
        let hl = self.reg.get_hl();
        let r = hl.wrapping_add(reg).wrapping_add(c);
        self.reg.flags.s = r & 0x8000 == 0x8000;
        self.reg.flags.z = r == 0x0000;
        self.reg.flags.h = (hl ^ reg ^ r) & 0x1000 != 0;
        self.reg.flags.p = ((hl ^ r) & (reg ^ r) & 0x8000) != 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (hl as u32) + (reg as u32 + c as u32) > 0x0000FFFF;
        self.reg.flags.b5 = r & 0b00100000_00000000 != 0;
        self.reg.flags.b3 = r & 0b00001000_00000000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        r
    }

    fn neg(&mut self) {
        let a = self.reg.a;
        let r = 0_u8.wrapping_sub(a);
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = (a ^ r) & 0x10 != 0;
        self.reg.flags.p = a == 0x80;
        self.reg.flags.n = true;
        self.reg.flags.c = a != 0;
        self.reg.a = r;
        self.reg.flags.b5 = r & 0b00100000 != 0;
        self.reg.flags.b3 = r & 0b00001000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn ldi<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_hl();
        let d = self.reg.get_de();
        let data = bus.read_memory(s);
        bus.write_memory(d, data);
        self.reg.set_hl(s.wrapping_add(1));
        self.reg.set_de(d.wrapping_add(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        let n = data.wrapping_add(self.reg.a);
        self.reg.flags.b5 = n & 0b00000010 != 0;
        self.reg.flags.b3 = n & 0b00001000 != 0;
        self.reg.flags.h = false;
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = false;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn ldd<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_hl();
        let d = self.reg.get_de();
        let data = bus.read_memory(s);
        bus.write_memory(d, data);
        self.reg.set_hl(s.wrapping_sub(1));
        self.reg.set_de(d.wrapping_sub(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        let n = data.wrapping_add(self.reg.a);
        self.reg.flags.b5 = n & 0b00000010 != 0;
        self.reg.flags.b3 = n & 0b00001000 != 0;
        self.reg.flags.h = false;
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = false;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn cpi<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_hl();
        let data = bus.read_memory(s);
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.set_hl(s.wrapping_add(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = (data ^ r ^ a) & 0x10 != 0;
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
        let n = a.wrapping_sub(data).wrapping_sub(self.reg.flags.h as u8);
        self.reg.flags.b5 = n & 0b00000010 != 0;
        self.reg.flags.b3 = n & 0b00001000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn cpd<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_hl();
        let data = bus.read_memory(s);
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.set_hl(s.wrapping_sub(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = (a ^ r ^ data) & 0x10 != 0;
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
        let n = a.wrapping_sub(data).wrapping_sub(self.reg.flags.h as u8);
        self.reg.flags.b5 = n & 0b00000010 != 0;
        self.reg.flags.b3 = n & 0b00001000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn ini<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_bc();
        let data = bus.read_io(s);
        let d = self.reg.get_hl();
        bus.write_memory(d, data);
        self.reg.set_hl(d.wrapping_add(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let k = data as u16 + self.reg.c.wrapping_add(1) as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn ind<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_bc();
        let data = bus.read_io(s);
        let d = self.reg.get_hl();
        bus.write_memory(d, data);
        self.reg.set_hl(d.wrapping_sub(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let k = data as u16 + self.reg.c.wrapping_sub(1) as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn outi<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_hl();
        let data = bus.read_memory(s);
        self.reg.set_hl(s.wrapping_add(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let d = self.reg.get_bc();
        bus.write_io(d, data);
        let k = data as u16 + self.reg.l as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn outd<B: SystemBus>(&mut self, bus: &mut B) {
        let s = self.reg.get_hl();
        let data = bus.read_memory(s);
        self.reg.set_hl(s.wrapping_sub(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let d = self.reg.get_bc();
        bus.write_io(d, data);
        let k = data as u16 + self.reg.l as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) ^ self.reg.b as u16).count_ones() % 2 == 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn ld_a_ri(&mut self, reg: u8) {
        self.reg.a = reg;
        self.reg.flags.s = reg & 0x80 == 0x80;
        self.reg.flags.z = reg == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = self.iff2;
        self.reg.flags.n = false;
        self.reg.flags.b5 = reg & 0b00100000 != 0;
        self.reg.flags.b3 = reg & 0b00001000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn rld<B: SystemBus>(&mut self, bus: &mut B) {
        let n = bus.read_memory(self.reg.get_hl());
        let a = self.reg.a;
        let tmp = a & 0x0F;
        let a = (a & 0xF0) | (n >> 4);
        let n = (n << 4) | tmp;
        self.reg.a = a;
        self.reg.flags.s = a & 0x80 == 0x80;
        self.reg.flags.z = a == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = a.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.b5 = a & 0b00100000 != 0;
        self.reg.flags.b3 = a & 0b00001000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        bus.write_memory(self.reg.get_hl(), n);
    }

    fn rrd<B: SystemBus>(&mut self, bus: &mut B) {
        let n = bus.read_memory(self.reg.get_hl());
        let a = self.reg.a;
        let tmp = a << 4;
        let a = (a & 0xF0) | (n & 0x0F);
        let n = (n >> 4) | tmp;
        self.reg.a = a;
        self.reg.flags.s = a & 0x80 == 0x80;
        self.reg.flags.z = a == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = a.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.b5 = a & 0b00100000 != 0;
        self.reg.flags.b3 = a & 0b00001000 != 0;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        bus.write_memory(self.reg.get_hl(), n);
    }

    pub fn ed_instructions<B: SystemBus>(&mut self, bus: &mut B) -> u8 {
        self.reg.inc_r();
        self.reg.inc_pc();
        let opcode = bus.read_memory(self.reg.pc);
        let mut cycles = CYCLES_ED[opcode as usize];

        match opcode {
            // IN r, (C)
            0x40 => self.reg.b = self.in_r_c(bus),
            0x48 => self.reg.c = self.in_r_c(bus),
            0x50 => self.reg.d = self.in_r_c(bus),
            0x58 => self.reg.e = self.in_r_c(bus),
            0x60 => self.reg.h = self.in_r_c(bus),
            0x68 => self.reg.l = self.in_r_c(bus),
            0x70 => _ = self.in_r_c(bus),
            0x78 => self.reg.a = self.in_r_c(bus),
            // OUT (C), r
            0x41 => self.out_c_r(self.reg.b, bus),
            0x49 => self.out_c_r(self.reg.c, bus),
            0x51 => self.out_c_r(self.reg.d, bus),
            0x59 => self.out_c_r(self.reg.e, bus),
            0x61 => self.out_c_r(self.reg.h, bus),
            0x69 => self.out_c_r(self.reg.l, bus),
            0x71 => self.out_c_r(0x00, bus),
            0x79 => self.out_c_r(self.reg.a, bus),
            // SBC HL, rr
            0x42 => {
                let hl = self.sbc_hl_rr(self.reg.get_bc());
                self.reg.set_hl(hl);
            }
            0x52 => {
                let hl = self.sbc_hl_rr(self.reg.get_de());
                self.reg.set_hl(hl);
            }
            0x62 => {
                let hl = self.sbc_hl_rr(self.reg.get_hl());
                self.reg.set_hl(hl);
            }
            0x72 => {
                let hl = self.sbc_hl_rr(self.reg.sp);
                self.reg.set_hl(hl);
            }
            // ADC HL, rr
            0x4A => {
                let hl = self.adc_hl_rr(self.reg.get_bc());
                self.reg.set_hl(hl);
            }
            0x5A => {
                let hl = self.adc_hl_rr(self.reg.get_de());
                self.reg.set_hl(hl);
            }
            0x6A => {
                let hl = self.adc_hl_rr(self.reg.get_hl());
                self.reg.set_hl(hl);
            }
            0x7A => {
                let hl = self.adc_hl_rr(self.reg.sp);
                self.reg.set_hl(hl);
            }
            // LD (nn), rr
            0x43 => {
                let nn = self.get_nn(bus);
                bus.write_memory(nn, self.reg.c);
                bus.write_memory(nn.wrapping_add(1), self.reg.b);
            }
            0x53 => {
                let nn = self.get_nn(bus);
                bus.write_memory(nn, self.reg.e);
                bus.write_memory(nn.wrapping_add(1), self.reg.d);
            }
            0x63 => {
                let nn = self.get_nn(bus);
                bus.write_memory(nn, self.reg.l);
                bus.write_memory(nn.wrapping_add(1), self.reg.h);
            }
            0x73 => {
                let nn = self.get_nn(bus);
                let [spl, sph] = self.reg.sp.to_le_bytes();
                bus.write_memory(nn, spl);
                bus.write_memory(nn.wrapping_add(1), sph);
            }
            // LD rr, (nn)
            0x4B => {
                let nn = self.get_nn(bus);
                self.reg.c = bus.read_memory(nn);
                self.reg.b = bus.read_memory(nn.wrapping_add(1));
            }
            0x5B => {
                let nn = self.get_nn(bus);
                self.reg.e = bus.read_memory(nn);
                self.reg.d = bus.read_memory(nn.wrapping_add(1));
            }
            0x6B => {
                let nn = self.get_nn(bus);
                self.reg.l = bus.read_memory(nn);
                self.reg.h = bus.read_memory(nn.wrapping_add(1));
            }
            0x7B => {
                let nn = self.get_nn(bus);
                let spl = bus.read_memory(nn);
                let sph = bus.read_memory(nn.wrapping_add(1));
                self.reg.sp = u16::from_le_bytes([spl, sph]);
            }
            0x44 | 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C => self.neg(),
            // Interrupt mode
            0x46 | 0x4E | 0x66 | 0x6E => {
                self.im = InterruptMode::IM_0;
                self.reg.flags.alu = false;
            }

            0x56 | 0x76 => {
                self.im = InterruptMode::IM_1;
                self.reg.flags.alu = false;
            }
            0x5E | 0x7E => {
                self.im = InterruptMode::IM_2;
                self.reg.flags.alu = false;
            }
            // LD I,A ; LD A,I ; LD R,A ; LD A,R
            0x47 => {
                self.reg.i = self.reg.a;
                self.reg.flags.alu = false;
            }
            0x57 => self.ld_a_ri(self.reg.i),
            0x4F => {
                self.reg.r = self.reg.a;
                self.reg.flags.alu = false;
            }
            0x5F => self.ld_a_ri(self.reg.r & 0x7F),
            // LDI ; LDIR
            0xA0 => self.ldi(bus),
            0xB0 => {
                self.ldi(bus);
                if self.reg.flags.p {
                    self.reg.dec_pc();
                    self.reg.flags.b5 = self.reg.pc & 0b00100000_00000000 != 0;
                    self.reg.flags.b3 = self.reg.pc & 0b00001000_00000000 != 0;
                    self.reg.dec_pc();
                    cycles += 5;
                }
            }
            // LDD ; LDDR
            0xA8 => self.ldd(bus),
            0xB8 => {
                self.ldd(bus);
                if self.reg.flags.p {
                    self.reg.dec_pc();
                    self.reg.flags.b5 = self.reg.pc & 0b00100000_00000000 != 0;
                    self.reg.flags.b3 = self.reg.pc & 0b00001000_00000000 != 0;
                    self.reg.dec_pc();
                    cycles += 5;
                }
            }
            // CPI ; CPIR
            0xA1 => self.cpi(bus),
            0xB1 => {
                self.cpi(bus);
                if self.reg.flags.p && !self.reg.flags.z {
                    self.reg.dec_pc();
                    self.reg.flags.b5 = self.reg.pc & 0b00100000_00000000 != 0;
                    self.reg.flags.b3 = self.reg.pc & 0b00001000_00000000 != 0;
                    self.reg.dec_pc();
                    cycles += 5;
                }
            }
            // CPD ; CPDR
            0xA9 => self.cpd(bus),
            0xB9 => {
                self.cpd(bus);
                if self.reg.flags.p && !self.reg.flags.z {
                    self.reg.dec_pc();
                    self.reg.flags.b5 = self.reg.pc & 0b00100000_00000000 != 0;
                    self.reg.flags.b3 = self.reg.pc & 0b00001000_00000000 != 0;
                    self.reg.dec_pc();
                    cycles += 5;
                }
            }
            // INI ; INIR
            0xA2 => self.ini(bus),
            0xB2 => {
                self.ini(bus);
                if !self.reg.flags.z {
                    self.reg.pc = self.reg.pc.wrapping_sub(2);
                    cycles += 5;
                }
            }
            // IND ; INDR
            0xAA => self.ind(bus),
            0xBA => {
                self.ind(bus);
                if !self.reg.flags.z {
                    self.reg.pc = self.reg.pc.wrapping_sub(2);
                    cycles += 5;
                }
            }
            // OUTI ; OUTIR
            0xA3 => self.outi(bus),
            0xB3 => {
                self.outi(bus);
                if !self.reg.flags.z {
                    self.reg.dec_pc();
                    self.reg.flags.b5 = self.reg.pc & 0b00100000_00000000 != 0;
                    self.reg.flags.b3 = self.reg.pc & 0b00001000_00000000 != 0;
                    self.reg.dec_pc();
                    cycles += 5;
                }
            }
            // OUTD ; OUTDR
            0xAB => self.outd(bus),
            0xBB => {
                self.outd(bus);
                if !self.reg.flags.z {
                    self.reg.dec_pc();
                    self.reg.flags.b5 = self.reg.pc & 0b00100000_00000000 != 0;
                    self.reg.flags.b3 = self.reg.pc & 0b00001000_00000000 != 0;
                    self.reg.dec_pc();
                    cycles += 5;
                }
            }
            // RETN
            0x45 | 0x55 | 0x5D | 0x65 | 0x6D | 0x75 | 0x7D => {
                self.iff1 = self.iff2;
                self.ret(bus);
                self.reg.flags.alu = false;
            }
            // RETI
            0x4D => {
                self.iff1 = self.iff2;
                self.ret(bus);
                self.reg.flags.alu = false;
            }

            // RRD and RLD
            0x67 => self.rrd(bus),
            0x6F => self.rld(bus),

            // NOP
            0x77 | 0x7F => self.reg.flags.alu = false,

            _ => {}
        }
        cycles
    }
}
