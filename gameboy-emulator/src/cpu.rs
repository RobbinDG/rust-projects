use crate::{MSB_MASK, REG_INTERRUPT_ENABLE, REG_INTERRUPT_FLAG};
use crate::addrreg::AddrReg;
use crate::condition::Condition;
use crate::dataloc::DataLoc;
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::reg::Reg;
use crate::register::Registers;

pub struct CPU {
    reg: Registers,
    mem: Memory,
    last: Instruction,
    /// Master interrupt enable (IME) flag
    ime: bool,
    ie_delay: i8,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            reg: Registers::new(),
            mem: Memory::new(rom),
            last: Instruction::NOP,
            ime: false,
            ie_delay: 0,
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.mem[self.reg.pc];
        self.reg.pc += 1;
        byte
    }

    fn next_byte_signed(&mut self) -> i8 {
        self.next_byte() as i8
    }

    pub fn step(&mut self) {
        // Fetch
        let byte = self.next_byte();
        println!("{:04x} {:02x}", self.reg.pc - 1, byte);

        // Decode
        let instruction = match byte {
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

            // INC 16-bit
            0x03 => Instruction::INC16(AddrReg::BC),
            0x13 => Instruction::INC16(AddrReg::DE),
            0x23 => Instruction::INC16(AddrReg::HL),
            0x33 => Instruction::INC16(AddrReg::SP),

            // DEC
            0x3D => Instruction::DEC(DataLoc::Reg(Reg::A)),
            0x05 => Instruction::DEC(DataLoc::Reg(Reg::B)),
            0x0D => Instruction::DEC(DataLoc::Reg(Reg::C)),
            0x15 => Instruction::DEC(DataLoc::Reg(Reg::D)),
            0x1D => Instruction::DEC(DataLoc::Reg(Reg::E)),
            0x25 => Instruction::DEC(DataLoc::Reg(Reg::H)),
            0x2D => Instruction::DEC(DataLoc::Reg(Reg::L)),
            0x35 => Instruction::DEC(DataLoc::AddrReg(AddrReg::HL)),

            // DEC 16-bit
            0x0B => Instruction::DEC16(AddrReg::BC),
            0x1B => Instruction::DEC16(AddrReg::DE),
            0x2B => Instruction::DEC16(AddrReg::HL),
            0x3B => Instruction::DEC16(AddrReg::SP),

            // 1. LD
            0x06 => Instruction::LD1(DataLoc::Reg(Reg::B), self.next_byte()),
            0x0E => Instruction::LD1(DataLoc::Reg(Reg::C), self.next_byte()),
            0x16 => Instruction::LD1(DataLoc::Reg(Reg::D), self.next_byte()),
            0x1E => Instruction::LD1(DataLoc::Reg(Reg::E), self.next_byte()),
            0x26 => Instruction::LD1(DataLoc::Reg(Reg::H), self.next_byte()),
            0x2E => Instruction::LD1(DataLoc::Reg(Reg::L), self.next_byte()),
            0x36 => Instruction::LD1(DataLoc::AddrReg(AddrReg::HL), self.next_byte()),

            // 2. LD
            0b01000000..0b01110110 | 0b01110111..=0b01111111 => {
                let first = (byte >> 3) & 0x7;
                let second = byte & 0x7;
                Instruction::LD2(Self::decode_register(first), Self::decode_register(second))
            }

            // 3. LD
            0x0A => Instruction::LD3(DataLoc::AddrReg(AddrReg::BC)),
            0x1A => Instruction::LD3(DataLoc::AddrReg(AddrReg::DE)),
            0xFA => Instruction::LD3(DataLoc::Addr(self.next_addr_lsb_first())),
            0x3E => Instruction::LD3(DataLoc::Value(self.next_byte())),

            // 4. LD
            0x02 => Instruction::LD4(DataLoc::AddrReg(AddrReg::BC)),
            0x12 => Instruction::LD4(DataLoc::AddrReg(AddrReg::DE)),
            0xEA => Instruction::LD4(DataLoc::Addr(self.next_addr_lsb_first())),

            // 5. LD
            0xF2 => Instruction::LD5,

            // 6. LD
            0xE2 => Instruction::LD6,

            // LDH
            0xE0 => Instruction::LDH1(self.next_byte()),
            0xF0 => Instruction::LDH2(self.next_byte()),

            // LDI
            0x2A => Instruction::LDI1,
            0x22 => Instruction::LDI2,

            // LD 16-bit
            0x01 => Instruction::LD16(AddrReg::BC, self.next_addr_lsb_first()),
            0x11 => Instruction::LD16(AddrReg::DE, self.next_addr_lsb_first()),
            0x21 => Instruction::LD16(AddrReg::HL, self.next_addr_lsb_first()),
            0x31 => Instruction::LD16(AddrReg::SP, self.next_addr_lsb_first()),
            0xF9 => Instruction::LDSPHL,
            0xF8 => Instruction::LDHL(self.next_byte_signed()),

            // PUSH
            0xF5 => Instruction::PUSH(AddrReg::AF),
            0xC5 => Instruction::PUSH(AddrReg::BC),
            0xD5 => Instruction::PUSH(AddrReg::DE),
            0xE5 => Instruction::PUSH(AddrReg::HL),

            // POP
            0xF1 => Instruction::POP(AddrReg::AF),
            0xC1 => Instruction::POP(AddrReg::BC),
            0xD1 => Instruction::POP(AddrReg::DE),
            0xE1 => Instruction::POP(AddrReg::HL),

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

            // AND
            0xA7 => Instruction::AND(DataLoc::Reg(Reg::A)),
            0xA0 => Instruction::AND(DataLoc::Reg(Reg::B)),
            0xA1 => Instruction::AND(DataLoc::Reg(Reg::C)),
            0xA2 => Instruction::AND(DataLoc::Reg(Reg::D)),
            0xA3 => Instruction::AND(DataLoc::Reg(Reg::E)),
            0xA4 => Instruction::AND(DataLoc::Reg(Reg::H)),
            0xA5 => Instruction::AND(DataLoc::Reg(Reg::L)),
            0xA6 => Instruction::AND(DataLoc::AddrReg(AddrReg::HL)),
            0xE6 => Instruction::AND(DataLoc::Value(self.next_byte())),

            // OR
            0xB7 => Instruction::OR(DataLoc::Reg(Reg::A)),
            0xB0 => Instruction::OR(DataLoc::Reg(Reg::B)),
            0xB1 => Instruction::OR(DataLoc::Reg(Reg::C)),
            0xB2 => Instruction::OR(DataLoc::Reg(Reg::D)),
            0xB3 => Instruction::OR(DataLoc::Reg(Reg::E)),
            0xB4 => Instruction::OR(DataLoc::Reg(Reg::H)),
            0xB5 => Instruction::OR(DataLoc::Reg(Reg::L)),
            0xB6 => Instruction::OR(DataLoc::AddrReg(AddrReg::HL)),
            0xF6 => Instruction::OR(DataLoc::Value(self.next_byte())),

            // XOR
            0xAF => Instruction::XOR(DataLoc::Reg(Reg::A)),
            0xA8 => Instruction::XOR(DataLoc::Reg(Reg::B)),
            0xA9 => Instruction::XOR(DataLoc::Reg(Reg::C)),
            0xAA => Instruction::XOR(DataLoc::Reg(Reg::D)),
            0xAB => Instruction::XOR(DataLoc::Reg(Reg::E)),
            0xAC => Instruction::XOR(DataLoc::Reg(Reg::H)),
            0xAD => Instruction::XOR(DataLoc::Reg(Reg::L)),
            0xAE => Instruction::XOR(DataLoc::AddrReg(AddrReg::HL)),
            0xEE => Instruction::XOR(DataLoc::Value(self.next_byte())),

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
            0xC2 => Instruction::JP2(Condition::NZ, self.next_addr_lsb_first()),
            0xCA => Instruction::JP2(Condition::Z, self.next_addr_lsb_first()),
            0xD2 => Instruction::JP2(Condition::NC, self.next_addr_lsb_first()),
            0xDA => Instruction::JP2(Condition::C, self.next_addr_lsb_first()),
            0xE9 => Instruction::JP3,

            // JR
            0x18 => Instruction::JR4(self.next_byte_signed()),
            0x20 => Instruction::JR5(Condition::NZ, self.next_byte_signed()),
            0x28 => Instruction::JR5(Condition::Z, self.next_byte_signed()),
            0x30 => Instruction::JR5(Condition::NC, self.next_byte_signed()),
            0x38 => Instruction::JR5(Condition::C, self.next_byte_signed()),

            // CALL
            0xCD => Instruction::CALL(self.next_addr_lsb_first()),
            0xC4 => Instruction::CALLc(Condition::NZ, self.next_addr_lsb_first()),
            0xCC => Instruction::CALLc(Condition::Z, self.next_addr_lsb_first()),
            0xD4 => Instruction::CALLc(Condition::NC, self.next_addr_lsb_first()),
            0xDC => Instruction::CALLc(Condition::C, self.next_addr_lsb_first()),

            // RET
            0xC9 => Instruction::RET,
            0xC0 => Instruction::RETc(Condition::NZ),
            0xC8 => Instruction::RETc(Condition::Z),
            0xD0 => Instruction::RETc(Condition::NC),
            0xD8 => Instruction::RETc(Condition::C),
            0xD9 => Instruction::RETI,

            // Misc
            0x27 => Instruction::DAA,
            0x2F => Instruction::CPL,
            0x3F => Instruction::CCF,
            0x37 => Instruction::SCF,
            0x00 => Instruction::NOP,
            0x76 => Instruction::HALT,
            0x10 => {
                assert_eq!(self.next_byte(), 0);
                Instruction::STOP
            }
            0xF3 => Instruction::DI,
            0xFB => Instruction::EI,

            0xCB => {
                let prefixed = self.next_byte();
                println!("{:02x}", prefixed);
                match prefixed {
                    // BIT: data is encoded as 0b01bbbrrr.
                    0b01000000..=0b01111111 => {
                        let b = (prefixed >> 3) & 0x07;
                        let r = prefixed & 0x07;
                        Instruction::BIT(b, Self::decode_register(r))
                    }
                    0x30..=0x37 => {
                        let r = prefixed & 0x07;
                        Instruction::SWAP(Self::decode_register(r))
                    }
                    // RES: data is encoded as 0b10bbbrrr.
                    0b10000000..=0b10111111 => {
                        let b = (prefixed >> 3) & 0x07;
                        let r = prefixed & 0x07;
                        Instruction::RES(b, Self::decode_register(r))
                    }


                    _ => todo!("Prefixed Op Code not implemented"),
                }
            }

            // RST
            0xC7 => Instruction::RST(0x00),
            0xCF => Instruction::RST(0x08),
            0xD7 => Instruction::RST(0x10),
            0xDF => Instruction::RST(0x18),
            0xE7 => Instruction::RST(0x20),
            0xEF => Instruction::RST(0x28),
            0xF7 => Instruction::RST(0x30),
            0xFF => Instruction::RST(0x38),

            _ => todo!("Op Code not implemented"),
        };

        println!("Decoded {:x?}", instruction);

        // Execute
        match instruction.clone() {
            Instruction::INC(l) => {
                let res = match l {
                    DataLoc::Reg(r) => {
                        let res = self.reg.get(r).wrapping_add(1);
                        self.reg.set(r, res);
                        res
                    }
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        self.mem[addr] = self.mem[addr].wrapping_add(1);
                        self.mem[addr]
                    }
                    _ => panic!("Not in instruction set."),
                };
                self.reg.set_flag(7, res == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, res.wrapping_sub(1) & 0x0F == 0x0F);
            }
            Instruction::INC16(r) => self.reg.set_pair(r, self.reg.get_pair(r).wrapping_add(1)),
            Instruction::DEC(l) => {
                let res = match l {
                    DataLoc::Reg(r) => {
                        let res = self.reg.get(r).wrapping_sub(1);
                        self.reg.set(r, res);
                        res
                    }
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        self.mem[addr] = self.mem[addr].wrapping_sub(1);
                        self.mem[addr]
                    }
                    _ => panic!("Not in instruction set."),
                };
                self.reg.set_flag(7, res == 0);
                self.reg.set_flag(6, true);
                self.reg.set_flag(5, res.wrapping_add(1) & 0x0F == 0x0F);
            }
            Instruction::DEC16(r) => self.reg.set_pair(r, self.reg.get_pair(r).wrapping_sub(1)),
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
            Instruction::JR4(addr) => {
                self.reg.pc = self.reg.pc.wrapping_add_signed(addr as i16);
            }
            Instruction::JR5(c, addr) => {
                println!("{}", self.reg.eval_condition(c));
                if self.reg.eval_condition(c) {
                    self.reg.pc = self.reg.pc.wrapping_add_signed(addr as i16);
                }
            }
            Instruction::LD1(r, v) => match r {
                DataLoc::Reg(r) => self.reg.set(r, v),
                DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)] = v,
                _ => panic!("Not in instruction set."),
            },
            Instruction::LD2(a, b) => {
                let v = match b {
                    DataLoc::Reg(r) => self.reg.get(r),
                    DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
                    _ => panic!("Not in instruction set."),
                };
                match a {
                    DataLoc::Reg(r) => self.reg.set(r, v),
                    DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)] = v,
                    _ => panic!("Not in instruction set."),
                }
            }
            Instruction::LD3(r) => {
                self.reg.a = match r {
                    DataLoc::Reg(r) => self.reg.get(r),
                    DataLoc::AddrReg(r) => self.mem[self.reg.get_pair(r)],
                    DataLoc::Addr(addr) => self.mem[addr],
                    DataLoc::Value(v) => v,
                }
            }
            Instruction::LD4(r) => match r {
                DataLoc::Reg(r) => self.reg.set(r, self.reg.a),
                DataLoc::AddrReg(r) => self.mem[self.reg.get_pair(r)] = self.reg.a,
                DataLoc::Addr(addr) => self.mem[addr] = self.reg.a,
                _ => panic!("Not in instruction set."),
            },
            Instruction::LD5 => {
                let addr = 0xFF00 | self.reg.c as u16;
                self.reg.a = self.mem[addr];
            }
            Instruction::LD6 => {
                let addr = 0xFF00 | self.reg.c as u16;
                self.mem[addr] = self.reg.a;
            }
            Instruction::LDH1(o) => {
                self.mem[0xFF00 | o as u16] = self.reg.a;
            }
            Instruction::LDH2(o) => {
                self.reg.a = self.mem[0xFF00 | o as u16];
            }
            Instruction::LDI1 => {
                self.reg.a = self.mem[self.reg.get_pair(AddrReg::HL)];
                self.reg
                    .set_pair(AddrReg::HL, self.reg.get_pair(AddrReg::HL) + 1);
            }
            Instruction::LDI2 => {
                self.mem[self.reg.get_pair(AddrReg::HL)] = self.reg.a;
                self.reg
                    .set_pair(AddrReg::HL, self.reg.get_pair(AddrReg::HL) + 1);
            }
            Instruction::LD16(reg, v) => {
                self.reg.set_pair(reg, v);
            }
            Instruction::LDSPHL => {
                self.reg
                    .set_pair(AddrReg::SP, self.reg.get_pair(AddrReg::HL));
            }
            Instruction::LDHL(offset) => {
                let r = self.reg.sp.wrapping_add_signed(offset as i16);
                self.reg.set_pair(AddrReg::HL, r);
                self.reg.set_flag(7, false);
                self.reg.set_flag(6, true);
                self.reg
                    .set_flag(5, ((self.reg.sp >> 3) | 1) ^ ((r >> 3) | 0x1) != 0);
                self.reg
                    .set_flag(4, ((self.reg.sp >> 7) | 1) ^ ((r >> 7) | 0x1) != 0);
            }
            Instruction::PUSH(r) => {
                self.push(self.reg.get_pair(r));
            }
            Instruction::POP(r) => {
                let val = self.pop();
                self.reg.set_pair(r, val);
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
            Instruction::AND(l) => {
                self.reg.a = self.and_set_flags(l);
            }
            Instruction::OR(l) => {
                self.reg.a = self.or_set_flags(l);
            }
            Instruction::XOR(l) => {
                self.reg.a = self.xor_set_flags(l);
            }
            Instruction::SWAP(l) => {
                let n = match l {
                    DataLoc::Reg(r) => self.reg.get(r),
                    DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
                    _ => panic!("Not in instruction set."),
                };
                let msn = n >> 4;
                let lsn = n & 0xF;
                let n_new = (lsn << 4) | msn;
                match l {
                    DataLoc::Reg(r) => self.reg.set(r, n_new),
                    DataLoc::AddrReg(AddrReg::HL) => {
                        self.mem[self.reg.get_pair(AddrReg::HL)] = n_new;
                    }
                    _ => panic!("Not in instruction set."),
                }
            }
            Instruction::CP(l) => {
                let _ = self.sub_set_flags(l);
            }
            Instruction::CALL(addr) => {
                self.call(addr);
            }
            Instruction::CALLc(cond, addr) => {
                if self.reg.eval_condition(cond) {
                    self.call(addr);
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
            Instruction::RETI => {
                self.ret();
                self.ime = true;
            }
            Instruction::BIT(data, n) => {
                let b = (data >> 3) & 0x7;
                if b > 7 {
                    panic!("Invalid RES bit: {b}");
                }
                let mask = (1 << b) as u8;
                let z = match n {
                    DataLoc::Reg(r) => self.reg.get(r) & mask,
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        self.mem[addr] & mask
                    }
                    _ => panic!("Not in instruction set."),
                };
                self.reg.set_flag(7, z == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, true);
            }
            Instruction::RES(data, n) => {
                let b = (data >> 3) & 0x7;
                if b > 7 {
                    panic!("Invalid RES bit: {b}");
                }
                let mask = !((1 << b) as u8);
                match n {
                    DataLoc::Reg(r) => self.reg.set(r, self.reg.get(r) & mask),
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        self.mem[addr] &= mask;
                    }
                    _ => panic!("Not in instruction set."),
                }
            }
            Instruction::DAA => {
                // http://www.z80.info/z80syntx.htm#DAA
                let c = self.reg.get_flag(4);
                let hi = self.reg.a >> 4;
                let h = self.reg.get_flag(5);
                let lo = self.reg.a & 0xF;
                let (added, c_after) = match self.last {
                    Instruction::ADD(_) | Instruction::ADC | Instruction::INC(_) => {
                        match (c, hi, h, lo) {
                            (false, 0x0..=0x9, false, 0x0..=0x9) => (0x00, false),
                            (false, 0x0..=0x8, false, 0xA..=0xF) => (0x06, false),
                            (false, 0x0..=0x9, true, 0x0..=0x3) => (0x06, false),
                            (false, 0xA..=0xF, false, 0x0..=0x9) => (0x60, true),
                            (false, 0x9..=0xF, false, 0xA..=0xF) => (0x66, true),
                            (false, 0xA..=0xF, true, 0x0..=0x3) => (0x66, true),
                            (true, 0x0..=0x2, false, 0x0..=0x9) => (0x60, true),
                            (true, 0x0..=0x2, false, 0xA..=0xF) => (0x66, true),
                            (true, 0x0..=0x3, true, 0x0..=0x3) => (0x66, true),
                            _ => panic!("Couldn't DAA convert"),
                        }
                    }
                    Instruction::SUB(_)
                    | Instruction::SBC
                    | Instruction::DEC(_)
                    | Instruction::NEG => match (c, hi, h, lo) {
                        (false, 0x0..=0x9, false, 0x0..=0x9) => (0x00, false),
                        (false, 0x0..=0x8, true, 0x6..=0xF) => (0xFA, false),
                        (true, 0x7..=0xF, false, 0x0..=0x9) => (0xA0, true),
                        (true, 0x6..=0xF, true, 0x6..=0xF) => (0x9A, true),
                        _ => panic!("Couldn't DAA convert"),
                    },
                    _ => panic!("DAA not supported for last instruction"),
                };
                self.reg.set_flag(7, self.reg.a == 0);
                self.reg.set_flag(5, false);
                self.reg.set_flag(4, c_after);
                self.reg.a = self.reg.a.wrapping_add(added);
            }
            Instruction::CPL => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(6, true);
                self.reg.set_flag(5, true);
            }
            Instruction::CCF => {
                self.reg.set_flag(4, !self.reg.get_flag(4));
            }
            Instruction::SCF => {
                self.reg.set_flag(4, true);
            }
            Instruction::NOP => println!("NOP"),
            Instruction::HALT => {
                todo!("Requires functioning interrupts")
            }
            Instruction::STOP => {
                todo!("Wait for button press")
            }
            Instruction::DI => self.ime = false,
            Instruction::EI => self.ie_delay = 1,
            Instruction::RST(proc) => {
                let curr = self.reg.pc - 1; // PC was incremented, decrement to get current
                self.push(curr);
                self.reg.pc = 0x0000 | proc as u16;
            }

            _ => panic!("Instruction not supported {:?}", instruction),
        }

        if self.ie_delay == 0 {
            self.ime = true;
        }
        if self.ie_delay >= 0 {
            self.ie_delay -= 1;
        }

        println!("Executed {:x?}", self.reg);

        self.last = instruction;
    }

    fn next_addr(&mut self) -> u16 {
        ((self.next_byte() as u16) << 8) | (self.next_byte() as u16)
    }

    fn next_addr_lsb_first(&mut self) -> u16 {
        (self.next_byte() as u16) | ((self.next_byte() as u16) << 8)
    }

    fn push(&mut self, val: u16) {
        let ls = (val & 0xFF) as u8;
        let ms = (val >> 8) as u8;
        self.mem[self.reg.sp] = ls;
        self.mem[self.reg.sp - 1] = ms;
        self.reg.sp -= 2;
    }

    fn pop(&mut self) -> u16 {
        let ls = self.mem[self.reg.sp + 2];
        let ms = self.mem[self.reg.sp + 1];
        self.reg.sp += 2;
        ((ms as u16) << 8) | (ls as u16)
    }

    fn call(&mut self, addr: u16) {
        let pc_next = self.reg.pc; // Program Counter is already at next instruction.
        self.push(pc_next);
        self.reg.pc = addr;
    }

    fn ret(&mut self) {
        self.reg.pc = self.pop();
    }

    fn add_set_flags(&mut self, l: DataLoc) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
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
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => panic!("Not in instruction set."),
        };
        let a = self.reg.a;
        let r = 0x0100 + a as u16 - n as u16;
        self.reg.set_flag(7, r == 0x0100);
        self.reg.set_flag(6, true);
        self.reg.set_flag(5, (a << 4) >= (n << 4));
        self.reg.set_flag(4, (r << 8) >= 0);
        (r << 8 >> 8) as u8
    }

    fn and_set_flags(&mut self, l: DataLoc) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => panic!("Not in instruction set."),
        };
        let r = self.reg.a & n;
        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, true);
        self.reg.set_flag(4, false);
        r
    }

    fn or_set_flags(&mut self, l: DataLoc) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => panic!("Not in instruction set."),
        };
        let r = self.reg.a | n;
        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, false);
        self.reg.set_flag(4, false);
        r
    }

    fn xor_set_flags(&mut self, l: DataLoc) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => self.mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => panic!("Not in instruction set."),
        };
        let r = self.reg.a ^ n;
        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, false);
        self.reg.set_flag(4, false);
        r
    }

    fn decode_register(encoding: u8) -> DataLoc {
        match encoding {
            0x7 => DataLoc::Reg(Reg::A),
            0x0 => DataLoc::Reg(Reg::B),
            0x1 => DataLoc::Reg(Reg::C),
            0x2 => DataLoc::Reg(Reg::D),
            0x3 => DataLoc::Reg(Reg::E),
            0x4 => DataLoc::Reg(Reg::H),
            0x5 => DataLoc::Reg(Reg::L),
            0x6 => DataLoc::AddrReg(AddrReg::HL),
            _ => panic!("Invalid register encoding."),
        }
    }

    fn get_if(&self, bit: u8) -> bool {
        if bit > 4 {
            panic!("Invalid IF bit.");
        }
        (self.mem[REG_INTERRUPT_FLAG] >> bit & 1) == 1
    }

    fn set_if(&mut self, bit: u8, value: bool) {
        if bit > 4 {
            panic!("Invalid IF bit.");
        }
        if value {
            self.mem[REG_INTERRUPT_FLAG] |= 1 << bit;
        } else {
            self.mem[REG_INTERRUPT_FLAG] &= !(1 << bit);
        }
    }

    fn get_ie(&self, bit: u8) -> bool {
        if bit > 4 {
            panic!("Invalid IE bit.");
        }
        (self.mem[REG_INTERRUPT_ENABLE] >> bit & 1) == 1
    }

    fn set_ie(&mut self, bit: u8, value: bool) {
        if bit > 4 {
            panic!("Invalid IE bit.");
        }
        if value {
            self.mem[REG_INTERRUPT_ENABLE] |= 1 << bit;
        } else {
            self.mem[REG_INTERRUPT_ENABLE] &= !(1 << bit);
        }
    }
}