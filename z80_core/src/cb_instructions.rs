use crate::{
    bus::SystemBus,
    cycles::{CYCLES_CB, CYCLES_DD_FD_CB},
    z80::*,
};

impl Z80 {
    fn rlc_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let r = data.rotate_left(1);
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x80) == 0x80;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn rrc_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let r = data.rotate_right(1);
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x01) == 0x01;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn rl_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let c = self.reg.flags.c as u8;
        let r = (data.rotate_left(1) & 0xFE) | c;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x80) == 0x80;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn rr_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let c = self.reg.flags.c as u8;
        let r = (data.rotate_right(1) & 0x7F) | c << 7;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x01) == 0x01;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn sla_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let r = data << 1;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x80) == 0x80;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn sra_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let r = ((data as i8) >> 1) as u8;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x01) == 0x01;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn sll_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let r = (data << 1) | 0x01;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x80) == 0x80;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn srl_r<B: SystemBus>(&mut self, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let r = data >> 1;
        self.reg.flags.s = false;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (data & 0x01) == 0x01;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        r
    }

    fn bit_b_r<B: SystemBus>(&mut self, bit: u8, reg: u8, d: u8, bus: &mut B) {
        let mask = 0x01_u8 << bit;
        let data = match self.p_inst {
            0xDD => {
                let addr = self.reg.get_ix().wrapping_add((d as i8) as u16);
                self.reg.flags.b5 = addr & 0b00100000_00000000 == 0b00100000_00000000;
                self.reg.flags.b3 = addr & 0b00001000_00000000 == 0b00001000_00000000;
                bus.read_memory(addr)
            }
            0xFD => {
                let addr = self.reg.get_iy().wrapping_add((d as i8) as u16);
                self.reg.flags.b5 = addr & 0b00100000_00000000 == 0b00100000_00000000;
                self.reg.flags.b3 = addr & 0b00001000_00000000 == 0b00001000_00000000;
                bus.read_memory(addr)
            }
            _ => {
                if d != 1 {
                    self.reg.flags.b5 = reg & 0b00100000 == 0b00100000;
                    self.reg.flags.b3 = reg & 0b00001000 == 0b00001000;
                } else {
                    let addr = self.reg.wz;
                    self.reg.flags.b5 = addr & 0b00100000_00000000 == 0b00100000_00000000;
                    self.reg.flags.b3 = addr & 0b00001000_00000000 == 0b00001000_00000000;
                }
                reg
            }
        };
        if bit == 7 {
            self.reg.flags.s = data & mask == mask;
        } else {
            self.reg.flags.s = false;
        }
        self.reg.flags.z = data & mask == 0x00;
        self.reg.flags.h = true;
        self.reg.flags.p = self.reg.flags.z;
        self.reg.flags.n = false;
        self.reg.flags.alu = self.reg.get_af() & 0x00FF != 0;
    }

    fn res_b_r<B: SystemBus>(&mut self, bit: u8, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let mut mask = 0x01_u8 << bit;
        mask = !mask;
        let r = data & mask;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        self.reg.flags.alu = false;
        r
    }

    fn set_b_r<B: SystemBus>(&mut self, bit: u8, reg: u8, d: u8, bus: &mut B) -> u8 {
        let data = match self.p_inst {
            0xDD => bus.read_memory(self.reg.get_ix().wrapping_add((d as i8) as u16)),
            0xFD => bus.read_memory(self.reg.get_iy().wrapping_add((d as i8) as u16)),
            _ => reg,
        };
        let mask = 0x01_u8 << bit;
        let r = data | mask;
        match self.p_inst {
            0xDD => bus.write_memory(self.reg.get_ix().wrapping_add((d as i8) as u16), r),
            0xFD => bus.write_memory(self.reg.get_iy().wrapping_add((d as i8) as u16), r),
            _ => {}
        }
        self.reg.flags.alu = false;
        r
    }

    pub fn cb_instructions<B: SystemBus>(&mut self, bus: &mut B) -> u8 {
        let d = if self.p_inst == 0xDD || self.p_inst == 0xFD {
            self.reg.inc_pc();
            bus.read_memory(self.reg.pc)
        } else {
            0_u8
        };
        self.reg.inc_r();
        self.reg.inc_pc();
        let opcode = bus.read_memory(self.reg.pc);
        let mut cycles = CYCLES_CB[opcode as usize];

        match opcode {
            // RLC r
            0x00 => self.reg.b = self.rlc_r(self.reg.b, d, bus),
            0x01 => self.reg.c = self.rlc_r(self.reg.c, d, bus),
            0x02 => self.reg.d = self.rlc_r(self.reg.d, d, bus),
            0x03 => self.reg.e = self.rlc_r(self.reg.e, d, bus),
            0x04 => self.reg.h = self.rlc_r(self.reg.h, d, bus),
            0x05 => self.reg.l = self.rlc_r(self.reg.l, d, bus),
            0x06 => {
                if self.p_inst != 0xDD || self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.rlc_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.rlc_r(0, d, bus);
                }
            }
            0x07 => self.reg.a = self.rlc_r(self.reg.a, d, bus),
            // RRC r
            0x08 => self.reg.b = self.rrc_r(self.reg.b, d, bus),
            0x09 => self.reg.c = self.rrc_r(self.reg.c, d, bus),
            0x0A => self.reg.d = self.rrc_r(self.reg.d, d, bus),
            0x0B => self.reg.e = self.rrc_r(self.reg.e, d, bus),
            0x0C => self.reg.h = self.rrc_r(self.reg.h, d, bus),
            0x0D => self.reg.l = self.rrc_r(self.reg.l, d, bus),
            0x0E => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.rrc_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.rrc_r(0, d, bus);
                }
            }
            0x0F => self.reg.a = self.rrc_r(self.reg.a, d, bus),
            // RL r
            0x10 => self.reg.b = self.rl_r(self.reg.b, d, bus),
            0x11 => self.reg.c = self.rl_r(self.reg.c, d, bus),
            0x12 => self.reg.d = self.rl_r(self.reg.d, d, bus),
            0x13 => self.reg.e = self.rl_r(self.reg.e, d, bus),
            0x14 => self.reg.h = self.rl_r(self.reg.h, d, bus),
            0x15 => self.reg.l = self.rl_r(self.reg.l, d, bus),
            0x16 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.rl_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.rl_r(0, d, bus);
                }
            }
            0x17 => self.reg.a = self.rl_r(self.reg.a, d, bus),
            // RR r
            0x18 => self.reg.b = self.rr_r(self.reg.b, d, bus),
            0x19 => self.reg.c = self.rr_r(self.reg.c, d, bus),
            0x1A => self.reg.d = self.rr_r(self.reg.d, d, bus),
            0x1B => self.reg.e = self.rr_r(self.reg.e, d, bus),
            0x1C => self.reg.h = self.rr_r(self.reg.h, d, bus),
            0x1D => self.reg.l = self.rr_r(self.reg.l, d, bus),
            0x1E => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.rr_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.rr_r(0, d, bus);
                }
            }
            0x1F => self.reg.a = self.rr_r(self.reg.a, d, bus),
            // SLA r
            0x20 => self.reg.b = self.sla_r(self.reg.b, d, bus),
            0x21 => self.reg.c = self.sla_r(self.reg.c, d, bus),
            0x22 => self.reg.d = self.sla_r(self.reg.d, d, bus),
            0x23 => self.reg.e = self.sla_r(self.reg.e, d, bus),
            0x24 => self.reg.h = self.sla_r(self.reg.h, d, bus),
            0x25 => self.reg.l = self.sla_r(self.reg.l, d, bus),
            0x26 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.sla_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.sla_r(0, d, bus);
                }
            }
            0x27 => self.reg.a = self.sla_r(self.reg.a, d, bus),
            // SRA r
            0x28 => self.reg.b = self.sra_r(self.reg.b, d, bus),
            0x29 => self.reg.c = self.sra_r(self.reg.c, d, bus),
            0x2A => self.reg.d = self.sra_r(self.reg.d, d, bus),
            0x2B => self.reg.e = self.sra_r(self.reg.e, d, bus),
            0x2C => self.reg.h = self.sra_r(self.reg.h, d, bus),
            0x2D => self.reg.l = self.sra_r(self.reg.l, d, bus),
            0x2E => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.sra_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.sra_r(0, d, bus);
                }
            }
            0x2F => self.reg.a = self.sra_r(self.reg.a, d, bus),
            // SLL r (undocumented)
            0x30 => self.reg.b = self.sll_r(self.reg.b, d, bus),
            0x31 => self.reg.c = self.sll_r(self.reg.c, d, bus),
            0x32 => self.reg.d = self.sll_r(self.reg.d, d, bus),
            0x33 => self.reg.e = self.sll_r(self.reg.e, d, bus),
            0x34 => self.reg.h = self.sll_r(self.reg.h, d, bus),
            0x35 => self.reg.l = self.sll_r(self.reg.l, d, bus),
            0x36 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.sll_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.sll_r(0, d, bus);
                }
            }
            0x37 => self.reg.a = self.sll_r(self.reg.a, d, bus),
            // SRL r
            0x38 => self.reg.b = self.srl_r(self.reg.b, d, bus),
            0x39 => self.reg.c = self.srl_r(self.reg.c, d, bus),
            0x3A => self.reg.d = self.srl_r(self.reg.d, d, bus),
            0x3B => self.reg.e = self.srl_r(self.reg.e, d, bus),
            0x3C => self.reg.h = self.srl_r(self.reg.h, d, bus),
            0x3D => self.reg.l = self.srl_r(self.reg.l, d, bus),
            0x3E => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.srl_r(data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.srl_r(0, d, bus);
                }
            }
            0x3F => self.reg.a = self.srl_r(self.reg.a, d, bus),
            // Bit b, r
            0x40 => self.bit_b_r(0, self.reg.b, d, bus),
            0x41 => self.bit_b_r(0, self.reg.c, d, bus),
            0x42 => self.bit_b_r(0, self.reg.d, d, bus),
            0x43 => self.bit_b_r(0, self.reg.e, d, bus),
            0x44 => self.bit_b_r(0, self.reg.h, d, bus),
            0x45 => self.bit_b_r(0, self.reg.l, d, bus),
            0x46 => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    0,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x47 => self.bit_b_r(0, self.reg.a, d, bus),
            0x48 => self.bit_b_r(1, self.reg.b, d, bus),
            0x49 => self.bit_b_r(1, self.reg.c, d, bus),
            0x4A => self.bit_b_r(1, self.reg.d, d, bus),
            0x4B => self.bit_b_r(1, self.reg.e, d, bus),
            0x4C => self.bit_b_r(1, self.reg.h, d, bus),
            0x4D => self.bit_b_r(1, self.reg.l, d, bus),
            0x4E => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    1,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x4F => self.bit_b_r(1, self.reg.a, d, bus),
            0x50 => self.bit_b_r(2, self.reg.b, d, bus),
            0x51 => self.bit_b_r(2, self.reg.c, d, bus),
            0x52 => self.bit_b_r(2, self.reg.d, d, bus),
            0x53 => self.bit_b_r(2, self.reg.e, d, bus),
            0x54 => self.bit_b_r(2, self.reg.h, d, bus),
            0x55 => self.bit_b_r(2, self.reg.l, d, bus),
            0x56 => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    2,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x57 => self.bit_b_r(2, self.reg.a, d, bus),
            0x58 => self.bit_b_r(3, self.reg.b, d, bus),
            0x59 => self.bit_b_r(3, self.reg.c, d, bus),
            0x5A => self.bit_b_r(3, self.reg.d, d, bus),
            0x5B => self.bit_b_r(3, self.reg.e, d, bus),
            0x5C => self.bit_b_r(3, self.reg.h, d, bus),
            0x5D => self.bit_b_r(3, self.reg.l, d, bus),
            0x5E => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    3,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x5F => self.bit_b_r(3, self.reg.a, d, bus),
            0x60 => self.bit_b_r(4, self.reg.b, d, bus),
            0x61 => self.bit_b_r(4, self.reg.c, d, bus),
            0x62 => self.bit_b_r(4, self.reg.d, d, bus),
            0x63 => self.bit_b_r(4, self.reg.e, d, bus),
            0x64 => self.bit_b_r(4, self.reg.h, d, bus),
            0x65 => self.bit_b_r(4, self.reg.l, d, bus),
            0x66 => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    4,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x67 => self.bit_b_r(4, self.reg.a, d, bus),
            0x68 => self.bit_b_r(5, self.reg.b, d, bus),
            0x69 => self.bit_b_r(5, self.reg.c, d, bus),
            0x6A => self.bit_b_r(5, self.reg.d, d, bus),
            0x6B => self.bit_b_r(5, self.reg.e, d, bus),
            0x6C => self.bit_b_r(5, self.reg.h, d, bus),
            0x6D => self.bit_b_r(5, self.reg.l, d, bus),
            0x6E => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    5,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x6F => self.bit_b_r(5, self.reg.a, d, bus),
            0x70 => self.bit_b_r(6, self.reg.b, d, bus),
            0x71 => self.bit_b_r(6, self.reg.c, d, bus),
            0x72 => self.bit_b_r(6, self.reg.d, d, bus),
            0x73 => self.bit_b_r(6, self.reg.e, d, bus),
            0x74 => self.bit_b_r(6, self.reg.h, d, bus),
            0x75 => self.bit_b_r(6, self.reg.l, d, bus),
            0x76 => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    6,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x77 => self.bit_b_r(6, self.reg.a, d, bus),
            0x78 => self.bit_b_r(7, self.reg.b, d, bus),
            0x79 => self.bit_b_r(7, self.reg.c, d, bus),
            0x7A => self.bit_b_r(7, self.reg.d, d, bus),
            0x7B => self.bit_b_r(7, self.reg.e, d, bus),
            0x7C => self.bit_b_r(7, self.reg.h, d, bus),
            0x7D => self.bit_b_r(7, self.reg.l, d, bus),
            0x7E => {
                let data = bus.read_memory(self.reg.get_hl());
                self.bit_b_r(
                    7,
                    data,
                    if self.p_inst != 0xDD && self.p_inst != 0xFD {
                        1
                    } else {
                        d
                    },
                    bus,
                );
            }
            0x7F => self.bit_b_r(7, self.reg.a, d, bus),
            // RES b, r
            0x80 => self.reg.b = self.res_b_r(0, self.reg.b, d, bus),
            0x81 => self.reg.c = self.res_b_r(0, self.reg.c, d, bus),
            0x82 => self.reg.d = self.res_b_r(0, self.reg.d, d, bus),
            0x83 => self.reg.e = self.res_b_r(0, self.reg.e, d, bus),
            0x84 => self.reg.h = self.res_b_r(0, self.reg.h, d, bus),
            0x85 => self.reg.l = self.res_b_r(0, self.reg.l, d, bus),
            0x86 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(0, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(0, 0, d, bus);
                }
            }
            0x87 => self.reg.a = self.res_b_r(0, self.reg.a, d, bus),
            0x88 => self.reg.b = self.res_b_r(1, self.reg.b, d, bus),
            0x89 => self.reg.c = self.res_b_r(1, self.reg.c, d, bus),
            0x8A => self.reg.d = self.res_b_r(1, self.reg.d, d, bus),
            0x8B => self.reg.e = self.res_b_r(1, self.reg.e, d, bus),
            0x8C => self.reg.h = self.res_b_r(1, self.reg.h, d, bus),
            0x8D => self.reg.l = self.res_b_r(1, self.reg.l, d, bus),
            0x8E => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(1, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(1, 0, d, bus);
                }
            }
            0x8F => self.reg.a = self.res_b_r(1, self.reg.a, d, bus),
            0x90 => self.reg.b = self.res_b_r(2, self.reg.b, d, bus),
            0x91 => self.reg.c = self.res_b_r(2, self.reg.c, d, bus),
            0x92 => self.reg.d = self.res_b_r(2, self.reg.d, d, bus),
            0x93 => self.reg.e = self.res_b_r(2, self.reg.e, d, bus),
            0x94 => self.reg.h = self.res_b_r(2, self.reg.h, d, bus),
            0x95 => self.reg.l = self.res_b_r(2, self.reg.l, d, bus),
            0x96 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(2, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(2, 0, d, bus);
                }
            }
            0x97 => self.reg.a = self.res_b_r(2, self.reg.a, d, bus),
            0x98 => self.reg.b = self.res_b_r(3, self.reg.b, d, bus),
            0x99 => self.reg.c = self.res_b_r(3, self.reg.c, d, bus),
            0x9A => self.reg.d = self.res_b_r(3, self.reg.d, d, bus),
            0x9B => self.reg.e = self.res_b_r(3, self.reg.e, d, bus),
            0x9C => self.reg.h = self.res_b_r(3, self.reg.h, d, bus),
            0x9D => self.reg.l = self.res_b_r(3, self.reg.l, d, bus),
            0x9E => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(3, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(3, 0, d, bus);
                }
            }
            0x9F => self.reg.a = self.res_b_r(3, self.reg.a, d, bus),
            0xA0 => self.reg.b = self.res_b_r(4, self.reg.b, d, bus),
            0xA1 => self.reg.c = self.res_b_r(4, self.reg.c, d, bus),
            0xA2 => self.reg.d = self.res_b_r(4, self.reg.d, d, bus),
            0xA3 => self.reg.e = self.res_b_r(4, self.reg.e, d, bus),
            0xA4 => self.reg.h = self.res_b_r(4, self.reg.h, d, bus),
            0xA5 => self.reg.l = self.res_b_r(4, self.reg.l, d, bus),
            0xA6 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(4, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(4, 0, d, bus);
                }
            }
            0xA7 => self.reg.a = self.res_b_r(4, self.reg.a, d, bus),
            0xA8 => self.reg.b = self.res_b_r(5, self.reg.b, d, bus),
            0xA9 => self.reg.c = self.res_b_r(5, self.reg.c, d, bus),
            0xAA => self.reg.d = self.res_b_r(5, self.reg.d, d, bus),
            0xAB => self.reg.e = self.res_b_r(5, self.reg.e, d, bus),
            0xAC => self.reg.h = self.res_b_r(5, self.reg.h, d, bus),
            0xAD => self.reg.l = self.res_b_r(5, self.reg.l, d, bus),
            0xAE => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(5, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(5, 0, d, bus);
                }
            }
            0xAF => self.reg.a = self.res_b_r(5, self.reg.a, d, bus),
            0xB0 => self.reg.b = self.res_b_r(6, self.reg.b, d, bus),
            0xB1 => self.reg.c = self.res_b_r(6, self.reg.c, d, bus),
            0xB2 => self.reg.d = self.res_b_r(6, self.reg.d, d, bus),
            0xB3 => self.reg.e = self.res_b_r(6, self.reg.e, d, bus),
            0xB4 => self.reg.h = self.res_b_r(6, self.reg.h, d, bus),
            0xB5 => self.reg.l = self.res_b_r(6, self.reg.l, d, bus),
            0xB6 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(6, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(6, 0, d, bus);
                }
            }
            0xB7 => self.reg.a = self.res_b_r(6, self.reg.a, d, bus),
            0xB8 => self.reg.b = self.res_b_r(7, self.reg.b, d, bus),
            0xB9 => self.reg.c = self.res_b_r(7, self.reg.c, d, bus),
            0xBA => self.reg.d = self.res_b_r(7, self.reg.d, d, bus),
            0xBB => self.reg.e = self.res_b_r(7, self.reg.e, d, bus),
            0xBC => self.reg.h = self.res_b_r(7, self.reg.h, d, bus),
            0xBD => self.reg.l = self.res_b_r(7, self.reg.l, d, bus),
            0xBE => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.res_b_r(7, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.res_b_r(7, 0, d, bus);
                }
            }
            0xBF => self.reg.a = self.res_b_r(7, self.reg.a, d, bus),
            // SET b, r
            0xC0 => self.reg.b = self.set_b_r(0, self.reg.b, d, bus),
            0xC1 => self.reg.c = self.set_b_r(0, self.reg.c, d, bus),
            0xC2 => self.reg.d = self.set_b_r(0, self.reg.d, d, bus),
            0xC3 => self.reg.e = self.set_b_r(0, self.reg.e, d, bus),
            0xC4 => self.reg.h = self.set_b_r(0, self.reg.h, d, bus),
            0xC5 => self.reg.l = self.set_b_r(0, self.reg.l, d, bus),
            0xC6 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(0, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(0, 0, d, bus);
                }
            }
            0xC7 => self.reg.a = self.set_b_r(0, self.reg.a, d, bus),
            0xC8 => self.reg.b = self.set_b_r(1, self.reg.b, d, bus),
            0xC9 => self.reg.c = self.set_b_r(1, self.reg.c, d, bus),
            0xCA => self.reg.d = self.set_b_r(1, self.reg.d, d, bus),
            0xCB => self.reg.e = self.set_b_r(1, self.reg.e, d, bus),
            0xCC => self.reg.h = self.set_b_r(1, self.reg.h, d, bus),
            0xCD => self.reg.l = self.set_b_r(1, self.reg.l, d, bus),
            0xCE => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(1, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(1, 0, d, bus);
                }
            }
            0xCF => self.reg.a = self.set_b_r(1, self.reg.a, d, bus),
            0xD0 => self.reg.b = self.set_b_r(2, self.reg.b, d, bus),
            0xD1 => self.reg.c = self.set_b_r(2, self.reg.c, d, bus),
            0xD2 => self.reg.d = self.set_b_r(2, self.reg.d, d, bus),
            0xD3 => self.reg.e = self.set_b_r(2, self.reg.e, d, bus),
            0xD4 => self.reg.h = self.set_b_r(2, self.reg.h, d, bus),
            0xD5 => self.reg.l = self.set_b_r(2, self.reg.l, d, bus),
            0xD6 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(2, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(2, 0, d, bus);
                }
            }
            0xD7 => self.reg.a = self.set_b_r(2, self.reg.a, d, bus),
            0xD8 => self.reg.b = self.set_b_r(3, self.reg.b, d, bus),
            0xD9 => self.reg.c = self.set_b_r(3, self.reg.c, d, bus),
            0xDA => self.reg.d = self.set_b_r(3, self.reg.d, d, bus),
            0xDB => self.reg.e = self.set_b_r(3, self.reg.e, d, bus),
            0xDC => self.reg.h = self.set_b_r(3, self.reg.h, d, bus),
            0xDD => self.reg.l = self.set_b_r(3, self.reg.l, d, bus),
            0xDE => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(3, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(3, 0, d, bus);
                }
            }
            0xDF => self.reg.a = self.set_b_r(3, self.reg.a, d, bus),
            0xE0 => self.reg.b = self.set_b_r(4, self.reg.b, d, bus),
            0xE1 => self.reg.c = self.set_b_r(4, self.reg.c, d, bus),
            0xE2 => self.reg.d = self.set_b_r(4, self.reg.d, d, bus),
            0xE3 => self.reg.e = self.set_b_r(4, self.reg.e, d, bus),
            0xE4 => self.reg.h = self.set_b_r(4, self.reg.h, d, bus),
            0xE5 => self.reg.l = self.set_b_r(4, self.reg.l, d, bus),
            0xE6 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(4, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(4, 0, d, bus);
                }
            }
            0xE7 => self.reg.a = self.set_b_r(4, self.reg.a, d, bus),
            0xE8 => self.reg.b = self.set_b_r(5, self.reg.b, d, bus),
            0xE9 => self.reg.c = self.set_b_r(5, self.reg.c, d, bus),
            0xEA => self.reg.d = self.set_b_r(5, self.reg.d, d, bus),
            0xEB => self.reg.e = self.set_b_r(5, self.reg.e, d, bus),
            0xEC => self.reg.h = self.set_b_r(5, self.reg.h, d, bus),
            0xED => self.reg.l = self.set_b_r(5, self.reg.l, d, bus),
            0xEE => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(5, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(5, 0, d, bus);
                }
            }
            0xEF => self.reg.a = self.set_b_r(5, self.reg.a, d, bus),
            0xF0 => self.reg.b = self.set_b_r(6, self.reg.b, d, bus),
            0xF1 => self.reg.c = self.set_b_r(6, self.reg.c, d, bus),
            0xF2 => self.reg.d = self.set_b_r(6, self.reg.d, d, bus),
            0xF3 => self.reg.e = self.set_b_r(6, self.reg.e, d, bus),
            0xF4 => self.reg.h = self.set_b_r(6, self.reg.h, d, bus),
            0xF5 => self.reg.l = self.set_b_r(6, self.reg.l, d, bus),
            0xF6 => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(6, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(6, 0, d, bus);
                }
            }
            0xF7 => self.reg.a = self.set_b_r(6, self.reg.a, d, bus),
            0xF8 => self.reg.b = self.set_b_r(7, self.reg.b, d, bus),
            0xF9 => self.reg.c = self.set_b_r(7, self.reg.c, d, bus),
            0xFA => self.reg.d = self.set_b_r(7, self.reg.d, d, bus),
            0xFB => self.reg.e = self.set_b_r(7, self.reg.e, d, bus),
            0xFC => self.reg.h = self.set_b_r(7, self.reg.h, d, bus),
            0xFD => self.reg.l = self.set_b_r(7, self.reg.l, d, bus),
            0xFE => {
                if self.p_inst != 0xDD && self.p_inst != 0xFD {
                    let mut data = bus.read_memory(self.reg.get_hl());
                    data = self.set_b_r(7, data, d, bus);
                    bus.write_memory(self.reg.get_hl(), data);
                } else {
                    _ = self.set_b_r(7, 0, d, bus);
                }
            }
            0xFF => self.reg.a = self.set_b_r(7, self.reg.a, d, bus),
        };
        if self.p_inst == 0xDD || self.p_inst == 0xFD {
            cycles += CYCLES_DD_FD_CB[opcode as usize];
        }
        cycles
    }
}
