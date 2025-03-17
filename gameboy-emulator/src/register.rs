use crate::addrreg::AddrReg;
use crate::condition::Condition;
use crate::reg::Reg;

#[derive(Debug, Clone)]
pub struct Registers {
    /// A: accumulator
    pub a: u8,
    /// F: flag register
    /// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
    /// | Z | N | H | C | 0 | 0 | 0 | 0 |
    /// Z : Zero flag
    /// N : Subtract flag
    /// H : Half carry flag
    /// C : Carry flag
    f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    /// Stack Pointer
    pub sp: u16,
    /// Program Counter
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            sp: 0xFFFE,
            pc: 0x0000,
            l: 0,
        }
    }

    pub fn get_flag(&self, flag: usize) -> bool {
        if flag < 4 || flag > 7 {
            // TODO check at compile time?
            panic!("Invalid flag: {}", flag)
        }
        ((self.f >> flag) & 1) != 0
    }

    pub fn set_flag(&mut self, flag: usize, value: bool) {
        if flag < 4 || flag > 7 {
            // TODO check at compile time?
            panic!("Invalid flag: {}", flag)
        }
        if value {
            self.f |= 1 << flag;
        } else {
            self.f &= !(1 << flag);
        }
    }

    pub fn get(&self, reg: Reg) -> u8 {
        match reg {
            Reg::A => self.a,
            Reg::B => self.b,
            Reg::C => self.c,
            Reg::D => self.d,
            Reg::E => self.e,
            Reg::H => self.h,
            Reg::L => self.l,
        }
    }

    pub fn set(&mut self, reg: Reg, val: u8) {
        match reg {
            Reg::A => self.a = val,
            Reg::B => self.b = val,
            Reg::C => self.c = val,
            Reg::D => self.d = val,
            Reg::E => self.e = val,
            Reg::H => self.h = val,
            Reg::L => self.l = val,
        }
    }

    pub fn get_pair(&self, reg: AddrReg) -> u16 {
        let (ms, ls) = match reg {
            AddrReg::BC => (self.b, self.c),
            AddrReg::DE => (self.d, self.e),
            AddrReg::HL => (self.h, self.l),
            AddrReg::AF => (self.a, self.f),
            _ => panic!("Not in instruction set."),
        };
        (ms as u16) << 8 | ls as u16
    }

    pub fn set_pair(&mut self, reg: AddrReg, value: u16) {
        let ms = (value >> 8) as u8;
        let ls = (value & 0xFF) as u8;
        match reg {
            AddrReg::BC => {
                self.b = ms;
                self.c = ls;
            }
            AddrReg::DE => {
                self.d = ms;
                self.e = ls;
            }
            AddrReg::HL => {
                self.h = ms;
                self.l = ls;
            }
            AddrReg::SP => {
                self.sp = value;
            }
            AddrReg::AF => {
                self.a = ms;
                self.f = ls;
            }
            _ => panic!("Not in instruction set."),
        };
    }

    pub fn eval_condition(&mut self, condition: Condition) -> bool {
        match condition {
            Condition::NZ => !self.get_flag(7),
            Condition::Z => self.get_flag(7),
            Condition::NC => !self.get_flag(4),
            Condition::C => self.get_flag(4),
        }
    }
}