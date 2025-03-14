use std::fs;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    /// Z reset
    NZ,
    /// Z set
    Z,
    /// C reset
    NC,
    /// C set
    C,
}

#[derive(Debug, Copy, Clone)]
pub enum Reg {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug)]
pub enum AddrReg {
    /// (BC)
    BC,
    /// (DE)
    DE,
    /// (HL)
    HL,
}

#[derive(Debug)]
pub enum DataLoc {
    Reg(Reg),
    AddrReg(AddrReg),
    Addr(u16),
    Value(u8),
}

#[derive(Debug)]
pub enum Instruction {
    NOP,
    INC(DataLoc),
    DEC(DataLoc),
    LD1(Reg, u8),
    LD2(Reg, Reg),
    LD3(DataLoc),
    LD4(DataLoc),
    LD5(Reg),
    LD6(Reg),
    LDH1(u8),
    LDH2(u8),
    LDI1,
    LDI2,
    ADD(DataLoc),
    SUB(DataLoc),
    CP(DataLoc),
    JP1(u16),
    JP2(Condition, u16),
    JP3,
    JR4(u8),
    JR5(Condition, u8),
    JPc(Condition, u16),
    RET,
    RETc(Condition),
    RETI,
    RLA,
    RLCA,
}

#[derive(Debug)]
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

const MSB_MASK: u8 = 0b10000000;
const LSB_MASK: u8 = 0b00000001;
const LS_BYTE_MASK: u16 = 0x00FF;
const MS_BYTE_MASK: u16 = 0xFF00;

impl Registers {
    fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            sp: 0xFFFE,
            pc: 0x0100,
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

struct CPU {
    reg: Registers,
    mem: [u8; 65536],
    rom: Vec<u8>,
}

impl CPU {
    fn new(rom: Vec<u8>) -> Self {
        Self {
            reg: Registers::new(),
            mem: [0; 65536],
            rom,
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.rom[self.reg.pc as usize];
        self.reg.pc += 1;
        byte
    }

    pub fn step(&mut self) {
        // Fetch
        let byte = self.next_byte();
        println!("{} {:02x}", self.reg.pc, byte);

        // Decode
        let instruction = match byte {
            0x00 => Instruction::NOP,
            0x07 => Instruction::RLCA,

            // INC
            0x3C => Instruction::INC(DataLoc::Reg(Reg::A)),
            0x04 => Instruction::INC(DataLoc::Reg(Reg::B)),
            0x0C => Instruction::INC(DataLoc::Reg(Reg::C)),
            0x14 => Instruction::INC(DataLoc::Reg(Reg::D)),
            0x1C => Instruction::INC(DataLoc::Reg(Reg::E)),
            0x24 => Instruction::INC(DataLoc::Reg(Reg::H)),
            0x2C => Instruction::INC(DataLoc::Reg(Reg::L)),
            0x34 => Instruction::INC(DataLoc::AddrReg(AddrReg::HL)),

            // DEC
            0x3D => Instruction::DEC(DataLoc::Reg(Reg::A)),
            0x05 => Instruction::DEC(DataLoc::Reg(Reg::B)),
            0x0D => Instruction::DEC(DataLoc::Reg(Reg::C)),
            0x15 => Instruction::DEC(DataLoc::Reg(Reg::D)),
            0x1D => Instruction::DEC(DataLoc::Reg(Reg::E)),
            0x25 => Instruction::DEC(DataLoc::Reg(Reg::H)),
            0x2D => Instruction::DEC(DataLoc::Reg(Reg::L)),
            0x35 => Instruction::DEC(DataLoc::AddrReg(AddrReg::HL)),

            // 3. LD
            0x7F => Instruction::LD3(DataLoc::Reg(Reg::A)),
            0x78 => Instruction::LD3(DataLoc::Reg(Reg::B)),
            0x79 => Instruction::LD3(DataLoc::Reg(Reg::C)),
            0x7A => Instruction::LD3(DataLoc::Reg(Reg::D)),
            0x7B => Instruction::LD3(DataLoc::Reg(Reg::E)),
            0x7C => Instruction::LD3(DataLoc::Reg(Reg::H)),
            0x7D => Instruction::LD3(DataLoc::Reg(Reg::L)),
            0x0A => Instruction::LD3(DataLoc::AddrReg(AddrReg::BC)),
            0x1A => Instruction::LD3(DataLoc::AddrReg(AddrReg::DE)),
            0x7E => Instruction::LD3(DataLoc::AddrReg(AddrReg::HL)),
            0xFA => Instruction::LD3(DataLoc::Addr(self.next_addr())),
            0x3E => Instruction::LD3(DataLoc::Value(self.next_byte())),

            // 4. LD
            0x47 => Instruction::LD4(DataLoc::Reg(Reg::B)),
            0x4F => Instruction::LD4(DataLoc::Reg(Reg::C)),
            0x57 => Instruction::LD4(DataLoc::Reg(Reg::D)),
            0x5F => Instruction::LD4(DataLoc::Reg(Reg::E)),
            0x67 => Instruction::LD4(DataLoc::Reg(Reg::H)),
            0x6F => Instruction::LD4(DataLoc::Reg(Reg::L)),
            0x02 => Instruction::LD4(DataLoc::AddrReg(AddrReg::BC)),
            0x12 => Instruction::LD4(DataLoc::AddrReg(AddrReg::DE)),
            0x77 => Instruction::LD4(DataLoc::AddrReg(AddrReg::HL)),
            0xEA => Instruction::LD4(DataLoc::Addr(self.next_addr())),

            // LDH
            0xE0 => Instruction::LDH1(self.next_byte()),
            0xF0 => Instruction::LDH2(self.next_byte()),

            // LDI
            0x2A => Instruction::LDI1,
            0x22 => Instruction::LDI2,

            // ADD
            0x87 => Instruction::ADD(DataLoc::Reg(Reg::A)),
            0x80 => Instruction::ADD(DataLoc::Reg(Reg::B)),
            0x81 => Instruction::ADD(DataLoc::Reg(Reg::C)),
            0x82 => Instruction::ADD(DataLoc::Reg(Reg::D)),
            0x83 => Instruction::ADD(DataLoc::Reg(Reg::E)),
            0x84 => Instruction::ADD(DataLoc::Reg(Reg::H)),
            0x85 => Instruction::ADD(DataLoc::Reg(Reg::L)),
            0x86 => Instruction::ADD(DataLoc::AddrReg(AddrReg::HL)),
            0xC6 => Instruction::ADD(DataLoc::Value(self.next_byte())),

            // SUB
            0x97 => Instruction::SUB(DataLoc::Reg(Reg::A)),
            0x90 => Instruction::SUB(DataLoc::Reg(Reg::B)),
            0x91 => Instruction::SUB(DataLoc::Reg(Reg::C)),
            0x92 => Instruction::SUB(DataLoc::Reg(Reg::D)),
            0x93 => Instruction::SUB(DataLoc::Reg(Reg::E)),
            0x94 => Instruction::SUB(DataLoc::Reg(Reg::H)),
            0x95 => Instruction::SUB(DataLoc::Reg(Reg::L)),
            0x96 => Instruction::SUB(DataLoc::AddrReg(AddrReg::HL)),
            0xD6 => Instruction::SUB(DataLoc::Value(self.next_byte())),

            // CP
            0xBF => Instruction::CP(DataLoc::Reg(Reg::A)),
            0xB8 => Instruction::CP(DataLoc::Reg(Reg::B)),
            0xB9 => Instruction::CP(DataLoc::Reg(Reg::C)),
            0xBA => Instruction::CP(DataLoc::Reg(Reg::D)),
            0xBB => Instruction::CP(DataLoc::Reg(Reg::E)),
            0xBC => Instruction::CP(DataLoc::Reg(Reg::H)),
            0xBD => Instruction::CP(DataLoc::Reg(Reg::L)),
            0xBE => Instruction::CP(DataLoc::AddrReg(AddrReg::HL)),
            0xFE => Instruction::CP(DataLoc::Value(self.next_byte())),

            // JP
            0xC3 => Instruction::JP1(self.next_addr_lsb_first()),
            0xC2 => Instruction::JP2(Condition::NZ, self.next_addr()),
            0xCA => Instruction::JP2(Condition::Z, self.next_addr()),
            0xD2 => Instruction::JP2(Condition::NC, self.next_addr()),
            0xDA => Instruction::JP2(Condition::C, self.next_addr()),
            0xE9 => Instruction::JP3,

            // JR
            0x18 => Instruction::JR4(self.next_byte()),
            0x20 => Instruction::JR5(Condition::NZ, self.next_byte()),
            0x28 => Instruction::JR5(Condition::Z, self.next_byte()),
            0x30 => Instruction::JR5(Condition::NC, self.next_byte()),
            0x38 => Instruction::JR5(Condition::C, self.next_byte()),

            // RET
            0xC9 => Instruction::RET,
            0xC0 => Instruction::RETc(Condition::NZ),
            0xC8 => Instruction::RETc(Condition::Z),
            0xD0 => Instruction::RETc(Condition::NC),
            0xD8 => Instruction::RETc(Condition::C),

            _ => todo!("Op Code not implemented"),
        };

        println!("Decoded {:?}", instruction);

        // Execute
        match instruction {
            Instruction::NOP => {
                println!("NOP");
            }
            Instruction::INC(l) => {
                let res = match l {
                    DataLoc::Reg(r) => {
                        let res = self.reg.get(r).wrapping_add(1);
                        self.reg.set(r, res);
                        res
                    }
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL) as usize;
                        self.mem[addr] = self.mem[addr].wrapping_add(1);
                        self.mem[addr]
                    }
                    _ => panic!("Not in instruction set."),
                };
                self.reg.set_flag(7, res == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, res.wrapping_sub(1) & 0x0F == 0x0F);
            }
            Instruction::DEC(l) => {
                let res = match l {
                    DataLoc::Reg(r) => {
                        let res = self.reg.get(r).wrapping_sub(1);
                        self.reg.set(r, res);
                        res
                    }
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL) as usize;
                        self.mem[addr] = self.mem[addr].wrapping_sub(1);
                        self.mem[addr]
                    }
                    _ => panic!("Not in instruction set."),
                };
                self.reg.set_flag(7, res == 0);
                self.reg.set_flag(6, true);
                self.reg.set_flag(5, res.wrapping_add(1) & 0x0F == 0x0F);
            }
            Instruction::JP1(addr) => {
                self.reg.pc = addr;
            }
            Instruction::JP2(c, addr) => {
                if self.reg.eval_condition(c) {
                    self.reg.pc = addr;
                }
            }
            Instruction::JP3 => {
                self.reg.pc = self.reg.get_pair(AddrReg::HL);
            }
            Instruction::LD3(r) => {
                self.reg.a = match r {
                    DataLoc::Reg(r) => self.reg.get(r),
                    DataLoc::AddrReg(r) => self.mem[self.reg.get_pair(r) as usize],
                    DataLoc::Addr(addr) => self.mem[addr as usize],
                    DataLoc::Value(v) => v,
                }
            }
            Instruction::JR4(addr) => {
                self.reg.pc += addr as u16;
            }
            Instruction::JR5(c, addr) => {
                println!("{}", self.reg.eval_condition(c));
                if self.reg.eval_condition(c) {
                    self.reg.pc += addr as u16;
                }
            }
            Instruction::RET => {
                self.ret();
            }
            Instruction::RETc(cond) => {
                if self.reg.eval_condition(cond) {
                    self.ret();
                }
            }
            Instruction::LD4(r) => match r {
                DataLoc::Reg(r) => self.reg.set(r, self.reg.a),
                DataLoc::AddrReg(r) => self.mem[self.reg.get_pair(r) as usize] = self.reg.a,
                DataLoc::Addr(_) => todo!(),
                _ => panic!("Not in instruction set."),
            },
            Instruction::LDH1(o) => {
                self.mem[(0xFF00 | o as u16) as usize] = self.reg.a;
            }
            Instruction::LDH2(o) => {
                self.reg.a = self.mem[(0xFF00 | o as u16) as usize];
            }
            Instruction::LDI1 => {
                self.reg.a = self.mem[self.reg.get_pair(AddrReg::HL) as usize];
                self.reg
                    .set_pair(AddrReg::HL, self.reg.get_pair(AddrReg::HL) + 1);
            }
            Instruction::LDI2 => {
                self.mem[self.reg.get_pair(AddrReg::HL) as usize] = self.reg.a;
                self.reg
                    .set_pair(AddrReg::HL, self.reg.get_pair(AddrReg::HL) + 1);
            }
            Instruction::RLA => {
                let a_old = self.reg.a;
                let a = a_old << 1 + self.reg.get_flag(4) as u8;
                self.reg.a = a;
                self.reg.set_flag(7, a == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, false);
                self.reg.set_flag(4, (a_old & MSB_MASK) != 0); // Set carry
            }
            Instruction::RLCA => {
                let a_old = self.reg.a;
                let a = a_old << 1;
                self.reg.a = a;
                self.reg.set_flag(7, a == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, false);
                self.reg.set_flag(4, (a_old & MSB_MASK) != 0); // Set carry
            }
            Instruction::ADD(l) => {
                self.reg.a = self.add_set_flags(l);
            }
            Instruction::SUB(l) => {
                self.reg.a = self.sub_set_flags(l);
            }
            Instruction::CP(l) => {
                let _ = self.sub_set_flags(l);
            }

            _ => panic!("Instruction not supported {:?}", instruction),
        }

        println!("Executed {:?}", self.reg);
    }

    fn next_addr(&mut self) -> u16 {
        ((self.next_byte() as u16) << 8) | (self.next_byte() as u16)
    }

    fn next_addr_lsb_first(&mut self) -> u16 {
        (self.next_byte() as u16) | ((self.next_byte() as u16) << 8)
    }

    fn ret(&mut self) {
        let ls = self.mem[self.reg.sp as usize];
        let ms = self.mem[(self.reg.sp - 1) as usize];
        self.reg.sp -= 2;
        self.reg.pc = ((ms as u16) << 8) | (ls as u16);
    }

    fn add_set_flags(&mut self, l: DataLoc) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL) as usize],
            DataLoc::Value(v) => v,
            _ => panic!("Not in instruction set."),
        };
        let a = self.reg.a;
        let r = a as u16 + n as u16;
        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, (a & 0x0F) + (n & 0x0F) > 0x0F);
        self.reg.set_flag(4, (r << 8) != 0);
        r as u8
    }

    fn sub_set_flags(&mut self, l: DataLoc) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL) as usize],
            DataLoc::Value(v) => v,
            _ => panic!("Not in instruction set."),
        };
        let a = self.reg.a;
        println!("{:016b} {:016b}", a, n);
        println!("{:016b} {:016b}", (a as u16) << 8, (n as u16) << 8);
        let r = 0x0100 + a as u16 - n as u16;
        self.reg.set_flag(7, r == 0x0100);
        self.reg.set_flag(6, true);
        self.reg.set_flag(5, (a << 4) >= (n << 4));
        self.reg.set_flag(4, (r << 8) >= 0);
        (r << 8 >> 8) as u8
    }
}

fn main() {
    let filename = "./Pokemon Red (UE) [S][!].gb";
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut rom = vec![0; metadata.len() as usize];
    f.read(&mut rom).expect("buffer overflow");

    let mut cpu = CPU::new(rom);

    for _ in 0usize..40 {
        cpu.step();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inc() {
        let rom = vec![0x3E, 0b1110, 0x3C, 0x3C, 0x3C];
        let mut cpu = CPU::new(rom);
        cpu.reg.pc = 0;

        assert_eq!(cpu.reg.a, 0);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b1110);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b1111);
        assert_eq!(cpu.reg.get_flag(7), false);
        assert_eq!(cpu.reg.get_flag(6), false);
        assert_eq!(cpu.reg.get_flag(5), false);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b10000);
        assert_eq!(cpu.reg.get_flag(7), false);
        assert_eq!(cpu.reg.get_flag(6), false);
        assert_eq!(cpu.reg.get_flag(5), true);;
        cpu.step();
        assert_eq!(cpu.reg.a, 0b10001);
        assert_eq!(cpu.reg.get_flag(7), false);
        assert_eq!(cpu.reg.get_flag(6), false);
        assert_eq!(cpu.reg.get_flag(5), false);
    }

    #[test]
    fn test_sub_eq() {
        let rom = vec![0x3E, 0b1110, 0x97];
        let mut cpu = CPU::new(rom);
        cpu.reg.pc = 0;

        assert_eq!(cpu.reg.a, 0);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b1110);
        cpu.step();

        assert_eq!(cpu.reg.a, 0);
        assert_eq!(cpu.reg.get_flag(7), true);
        assert_eq!(cpu.reg.get_flag(6), true);
        assert_eq!(cpu.reg.get_flag(5), true);
        assert_eq!(cpu.reg.get_flag(4), true);
    }
}
