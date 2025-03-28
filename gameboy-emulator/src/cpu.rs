use crate::addrreg::AddrReg;
use crate::condition::Condition;
use crate::dataloc::DataLoc;
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::reg::Reg;
use crate::register::Registers;
use std::collections::VecDeque;

const MSB_MASK: u8 = 0b1000_0000;
const LSB_MASK: u8 = 0b0000_0001;
const INTERRUPT_HANDLERS: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

struct ExecLog {
    pc: u16,
    byte: u8,
    instruction: Instruction,
    registers: Registers,
}

pub struct CPU {
    pub reg: Registers,
    last: Instruction,
    /// Master interrupt enable (IME) flag
    ime: bool,
    ie_delay: i8,
    /// Debug
    dbg_exec_log: VecDeque<ExecLog>,
    halted: bool,
    breakpoint_delay: usize,
    instructions_out_of_interrupt: usize,
    pub instructions_count: [usize; 2 * 256],
    m_cycle_counter: u32,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            last: Instruction::NOP,
            ime: false,
            ie_delay: -1,
            dbg_exec_log: VecDeque::new(),
            halted: false,
            breakpoint_delay: 0,
            instructions_out_of_interrupt: 0,
            instructions_count: [0; 2 * 256],
            m_cycle_counter: 0,
        }
    }

    pub fn check_interrupts(&mut self, mem: &mut Memory) {
        let ie = mem[0xFFFF];
        let if_ = mem[0xFF0F];

        if ie & if_ > 0 {
            self.halted = false;
        }

        if !self.ime {
            return;
        }

        for b in 0..5 {
            if (if_ >> b) & 1 != 0 && (ie >> b) & 1 != 0 {
                mem[0xFF0F] &= !(1 << b);
                if self.ime {
                    self.ime = false;
                    self.call(INTERRUPT_HANDLERS[b], mem);
                    self.m_cycle_counter += 5;
                }
                return;
            }
        }
    }

    fn next_byte(&mut self, mem: &mut Memory) -> u8 {
        if !matches!(self.reg.pc, 0..0x7FFF | 0xFF80..=0xFFFE) {
            // self.cpu_crash("PC escaped valid code".to_string())
        }
        let byte = mem[self.reg.pc];
        self.reg.pc += 1;
        byte
    }

    fn next_byte_signed(&mut self, mem: &mut Memory) -> i8 {
        self.next_byte(mem) as i8
    }

    pub fn run_cycle(&mut self, mem: &mut Memory) -> u32 {
        if self.halted {
            self.update_timer(mem, 1);
            return 1;
        }

        // Fetch
        let byte = self.next_byte(mem);
        let dbg_current_pc = self.reg.pc - 1;

        if dbg_current_pc == 0x02f5 {
            // self.breakpoint_delay = 100;
        }

        // Decode
        let instruction = match byte {
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
            0x06 => Instruction::LD(DataLoc::Reg(Reg::B), DataLoc::Value(self.next_byte(mem))),
            0x0E => Instruction::LD(DataLoc::Reg(Reg::C), DataLoc::Value(self.next_byte(mem))),
            0x16 => Instruction::LD(DataLoc::Reg(Reg::D), DataLoc::Value(self.next_byte(mem))),
            0x1E => Instruction::LD(DataLoc::Reg(Reg::E), DataLoc::Value(self.next_byte(mem))),
            0x26 => Instruction::LD(DataLoc::Reg(Reg::H), DataLoc::Value(self.next_byte(mem))),
            0x2E => Instruction::LD(DataLoc::Reg(Reg::L), DataLoc::Value(self.next_byte(mem))),
            0x36 => Instruction::LD(
                DataLoc::AddrReg(AddrReg::HL),
                DataLoc::Value(self.next_byte(mem)),
            ),

            // 2. LD
            0b01000000..0b01110110 | 0b01110111..=0b01111111 => {
                let first = (byte >> 3) & 0x7;
                let second = byte & 0x7;
                Instruction::LD(Self::decode_register(first), Self::decode_register(second))
            }

            // 3. LD
            0x0A => Instruction::LD(DataLoc::Reg(Reg::A), DataLoc::AddrReg(AddrReg::BC)),
            0x1A => Instruction::LD(DataLoc::Reg(Reg::A), DataLoc::AddrReg(AddrReg::DE)),
            0xFA => Instruction::LD(
                DataLoc::Reg(Reg::A),
                DataLoc::Addr(self.next_addr_lsb_first(mem)),
            ),
            0x3E => Instruction::LD(DataLoc::Reg(Reg::A), DataLoc::Value(self.next_byte(mem))),

            // 4. LD
            0x02 => Instruction::LD(DataLoc::AddrReg(AddrReg::BC), DataLoc::Reg(Reg::A)),
            0x12 => Instruction::LD(DataLoc::AddrReg(AddrReg::DE), DataLoc::Reg(Reg::A)),
            0xEA => Instruction::LD(
                DataLoc::Addr(self.next_addr_lsb_first(mem)),
                DataLoc::Reg(Reg::A),
            ),

            // 5. LD
            0xF2 => Instruction::LD5,

            // 6. LD
            0xE2 => Instruction::LD6,

            // LDH
            0xE0 => Instruction::LDH1(self.next_byte(mem)),
            0xF0 => Instruction::LDH2(self.next_byte(mem)),

            // LDI
            0x2A => Instruction::LDI(DataLoc::Reg(Reg::A), DataLoc::AddrReg(AddrReg::HL)),
            0x22 => Instruction::LDI(DataLoc::AddrReg(AddrReg::HL), DataLoc::Reg(Reg::A)),

            // LDD
            0x3A => Instruction::LDD(DataLoc::Reg(Reg::A), DataLoc::AddrReg(AddrReg::HL)),
            0x32 => Instruction::LDD(DataLoc::AddrReg(AddrReg::HL), DataLoc::Reg(Reg::A)),

            // LD 16-bit
            0x01 => Instruction::LD16(AddrReg::BC, self.next_addr_lsb_first(mem)),
            0x11 => Instruction::LD16(AddrReg::DE, self.next_addr_lsb_first(mem)),
            0x21 => Instruction::LD16(AddrReg::HL, self.next_addr_lsb_first(mem)),
            0x31 => Instruction::LD16(AddrReg::SP, self.next_addr_lsb_first(mem)),
            0xF9 => Instruction::LDSPHL,
            0xF8 => Instruction::LDHL(self.next_byte_signed(mem)),
            0x08 => Instruction::LDnn(self.next_addr_lsb_first(mem)),

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
            0xC6 => Instruction::ADD(DataLoc::Value(self.next_byte(mem))),

            // ADC
            0x8F => Instruction::ADC(DataLoc::Reg(Reg::A)),
            0x88 => Instruction::ADC(DataLoc::Reg(Reg::B)),
            0x89 => Instruction::ADC(DataLoc::Reg(Reg::C)),
            0x8A => Instruction::ADC(DataLoc::Reg(Reg::D)),
            0x8B => Instruction::ADC(DataLoc::Reg(Reg::E)),
            0x8C => Instruction::ADC(DataLoc::Reg(Reg::H)),
            0x8D => Instruction::ADC(DataLoc::Reg(Reg::L)),
            0x8E => Instruction::ADC(DataLoc::AddrReg(AddrReg::HL)),
            0xCE => Instruction::ADC(DataLoc::Value(self.next_byte(mem))),

            // ADD 16-bit
            0x09 => Instruction::ADD16(AddrReg::BC),
            0x19 => Instruction::ADD16(AddrReg::DE),
            0x29 => Instruction::ADD16(AddrReg::HL),
            0x39 => Instruction::ADD16(AddrReg::SP),
            0xE8 => Instruction::ADD16n(self.next_byte_signed(mem)),

            // SUB
            0x9F => Instruction::SBC(DataLoc::Reg(Reg::A)),
            0x98 => Instruction::SBC(DataLoc::Reg(Reg::B)),
            0x99 => Instruction::SBC(DataLoc::Reg(Reg::C)),
            0x9A => Instruction::SBC(DataLoc::Reg(Reg::D)),
            0x9B => Instruction::SBC(DataLoc::Reg(Reg::E)),
            0x9C => Instruction::SBC(DataLoc::Reg(Reg::H)),
            0x9D => Instruction::SBC(DataLoc::Reg(Reg::L)),
            0x9E => Instruction::SBC(DataLoc::AddrReg(AddrReg::HL)),
            0xDE => Instruction::SBC(DataLoc::Value(self.next_byte(mem))),

            // SBC
            0x97 => Instruction::SUB(DataLoc::Reg(Reg::A)),
            0x90 => Instruction::SUB(DataLoc::Reg(Reg::B)),
            0x91 => Instruction::SUB(DataLoc::Reg(Reg::C)),
            0x92 => Instruction::SUB(DataLoc::Reg(Reg::D)),
            0x93 => Instruction::SUB(DataLoc::Reg(Reg::E)),
            0x94 => Instruction::SUB(DataLoc::Reg(Reg::H)),
            0x95 => Instruction::SUB(DataLoc::Reg(Reg::L)),
            0x96 => Instruction::SUB(DataLoc::AddrReg(AddrReg::HL)),
            0xD6 => Instruction::SUB(DataLoc::Value(self.next_byte(mem))),

            // AND
            0xA7 => Instruction::AND(DataLoc::Reg(Reg::A)),
            0xA0 => Instruction::AND(DataLoc::Reg(Reg::B)),
            0xA1 => Instruction::AND(DataLoc::Reg(Reg::C)),
            0xA2 => Instruction::AND(DataLoc::Reg(Reg::D)),
            0xA3 => Instruction::AND(DataLoc::Reg(Reg::E)),
            0xA4 => Instruction::AND(DataLoc::Reg(Reg::H)),
            0xA5 => Instruction::AND(DataLoc::Reg(Reg::L)),
            0xA6 => Instruction::AND(DataLoc::AddrReg(AddrReg::HL)),
            0xE6 => Instruction::AND(DataLoc::Value(self.next_byte(mem))),

            // OR
            0xB7 => Instruction::OR(DataLoc::Reg(Reg::A)),
            0xB0 => Instruction::OR(DataLoc::Reg(Reg::B)),
            0xB1 => Instruction::OR(DataLoc::Reg(Reg::C)),
            0xB2 => Instruction::OR(DataLoc::Reg(Reg::D)),
            0xB3 => Instruction::OR(DataLoc::Reg(Reg::E)),
            0xB4 => Instruction::OR(DataLoc::Reg(Reg::H)),
            0xB5 => Instruction::OR(DataLoc::Reg(Reg::L)),
            0xB6 => Instruction::OR(DataLoc::AddrReg(AddrReg::HL)),
            0xF6 => Instruction::OR(DataLoc::Value(self.next_byte(mem))),

            // XOR
            0xAF => Instruction::XOR(DataLoc::Reg(Reg::A)),
            0xA8 => Instruction::XOR(DataLoc::Reg(Reg::B)),
            0xA9 => Instruction::XOR(DataLoc::Reg(Reg::C)),
            0xAA => Instruction::XOR(DataLoc::Reg(Reg::D)),
            0xAB => Instruction::XOR(DataLoc::Reg(Reg::E)),
            0xAC => Instruction::XOR(DataLoc::Reg(Reg::H)),
            0xAD => Instruction::XOR(DataLoc::Reg(Reg::L)),
            0xAE => Instruction::XOR(DataLoc::AddrReg(AddrReg::HL)),
            0xEE => Instruction::XOR(DataLoc::Value(self.next_byte(mem))),

            // CP
            0xBF => Instruction::CP(DataLoc::Reg(Reg::A)),
            0xB8 => Instruction::CP(DataLoc::Reg(Reg::B)),
            0xB9 => Instruction::CP(DataLoc::Reg(Reg::C)),
            0xBA => Instruction::CP(DataLoc::Reg(Reg::D)),
            0xBB => Instruction::CP(DataLoc::Reg(Reg::E)),
            0xBC => Instruction::CP(DataLoc::Reg(Reg::H)),
            0xBD => Instruction::CP(DataLoc::Reg(Reg::L)),
            0xBE => Instruction::CP(DataLoc::AddrReg(AddrReg::HL)),
            0xFE => Instruction::CP(DataLoc::Value(self.next_byte(mem))),

            // JP
            0xC3 => Instruction::JP1(self.next_addr_lsb_first(mem)),
            0xC2 => Instruction::JP2(Condition::NZ, self.next_addr_lsb_first(mem)),
            0xCA => Instruction::JP2(Condition::Z, self.next_addr_lsb_first(mem)),
            0xD2 => Instruction::JP2(Condition::NC, self.next_addr_lsb_first(mem)),
            0xDA => Instruction::JP2(Condition::C, self.next_addr_lsb_first(mem)),
            0xE9 => Instruction::JP3,

            // JR
            0x18 => Instruction::JR4(self.next_byte_signed(mem)),
            0x20 => Instruction::JR5(Condition::NZ, self.next_byte_signed(mem)),
            0x28 => Instruction::JR5(Condition::Z, self.next_byte_signed(mem)),
            0x30 => Instruction::JR5(Condition::NC, self.next_byte_signed(mem)),
            0x38 => Instruction::JR5(Condition::C, self.next_byte_signed(mem)),

            // CALL
            0xCD => Instruction::CALL(self.next_addr_lsb_first(mem)),
            0xC4 => Instruction::CALLc(Condition::NZ, self.next_addr_lsb_first(mem)),
            0xCC => Instruction::CALLc(Condition::Z, self.next_addr_lsb_first(mem)),
            0xD4 => Instruction::CALLc(Condition::NC, self.next_addr_lsb_first(mem)),
            0xDC => Instruction::CALLc(Condition::C, self.next_addr_lsb_first(mem)),

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
                assert_eq!(self.next_byte(mem), 0);
                Instruction::STOP
            }
            0xF3 => Instruction::DI,
            0xFB => Instruction::EI,

            // Rotates & Shifts
            0x07 => Instruction::RLCA,
            0x17 => Instruction::RLA,
            0x0F => Instruction::RRCA,
            0x1F => Instruction::RRA,

            0xCB => {
                let prefixed = self.next_byte(mem);
                self.instructions_count[256 + prefixed as usize] += 1;
                match prefixed {
                    // Rotates & Shifts
                    0x00..=0x07 => {
                        let r = prefixed & 0x07;
                        Instruction::RLC(Self::decode_register(r))
                    }
                    0x08..=0x0F => {
                        let r = prefixed & 0x07;
                        Instruction::RRC(Self::decode_register(r))
                    }
                    0x10..=0x17 => {
                        let r = prefixed & 0x07;
                        Instruction::RL(Self::decode_register(r))
                    }
                    0x18..=0x1F => {
                        let r = prefixed & 0x07;
                        Instruction::RR(Self::decode_register(r))
                    }
                    0x20..=0x27 => {
                        let r = prefixed & 0x07;
                        Instruction::SLA(Self::decode_register(r))
                    }
                    0x28..=0x2F => {
                        let r = prefixed & 0x07;
                        Instruction::SRA(Self::decode_register(r))
                    }
                    0x38..=0x3F => {
                        let r = prefixed & 0x07;
                        Instruction::SRL(Self::decode_register(r))
                    }
                    0x30..=0x37 => {
                        let r = prefixed & 0x07;
                        Instruction::SWAP(Self::decode_register(r))
                    }
                    // BIT: data is encoded as 0b01bbbrrr.
                    0x40..=0x7F => {
                        let b = (prefixed >> 3) & 0b0000_0111;
                        let r = prefixed & 0b0000_0111;
                        Instruction::BIT(b, Self::decode_register(r))
                    }
                    // RES: data is encoded as 0b10bbbrrr.
                    0b10000000..=0b10111111 => {
                        let b = (prefixed >> 3) & 0x07;
                        let r = prefixed & 0x07;
                        Instruction::RES(b, Self::decode_register(r))
                    }
                    0b11000000..=0b11111111 => {
                        let b = (prefixed >> 3) & 0x07;
                        let r = prefixed & 0x07;
                        Instruction::SET(b, Self::decode_register(r))
                    }
                    _ => todo!("Prefixed Op Code {:02x} not implemented", prefixed),
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

            _ => {
                self.print_exec_log();
                todo!("Op Code {:02x} not implemented", byte)
            }
        };
        self.instructions_count[byte as usize] += 1;

        // Execute
        let mut conditional_extra_cycles = 0u32;
        match instruction.clone() {
            Instruction::INC(l) => {
                let (old, res) = match l {
                    DataLoc::Reg(r) => {
                        let old = self.reg.get(r);
                        let res = old.wrapping_add(1);
                        self.reg.set(r, res);
                        (old, res)
                    }
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        let old = mem[addr];
                        mem[addr] = old.wrapping_add(1);
                        (old, mem[addr])
                    }
                    _ => self.cpu_crash("Not in instruction set.".to_string()),
                };
                self.reg.set_flag(7, res == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, old & 0x0F == 0x0F);
            }
            Instruction::INC16(r) => self.reg.set_pair(r, self.reg.get_pair(r).wrapping_add(1)),
            Instruction::DEC(l) => {
                let (_, res) = match l {
                    DataLoc::Reg(r) => {
                        let old = self.reg.get(r);
                        let res = old.wrapping_sub(1);
                        self.reg.set(r, res);
                        (old, res)
                    }
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        let old = mem[addr];
                        mem[addr] = old.wrapping_sub(1);
                        (old, mem[addr])
                    }
                    _ => self.cpu_crash("Not in instruction set.".to_string()),
                };
                self.reg.set_flag(7, res == 0);
                self.reg.set_flag(6, true);
                self.reg.set_flag(5, res & 0x0F == 0x0F);
            }
            Instruction::DEC16(r) => self.reg.set_pair(r, self.reg.get_pair(r).wrapping_sub(1)),
            Instruction::JP1(addr) => {
                self.reg.pc = addr;
            }
            Instruction::JP2(c, addr) => {
                if self.reg.eval_condition(c) {
                    conditional_extra_cycles = 1;
                    if addr == 0xc1b9 {
                        println!("FAILED TEST");
                    }
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
                if self.reg.eval_condition(c) {
                    conditional_extra_cycles = 1;
                    self.reg.pc = self.reg.pc.wrapping_add_signed(addr as i16);
                }
            }
            Instruction::LD(a, b) => {
                self.ld_8_bit(a, b, mem);
            }
            Instruction::LD5 => {
                self.ld_8_bit(
                    DataLoc::Reg(Reg::A),
                    DataLoc::Addr(0xFF00 | self.reg.c as u16),
                    mem,
                );
            }
            Instruction::LD6 => {
                self.ld_8_bit(
                    DataLoc::Addr(0xFF00 | self.reg.c as u16),
                    DataLoc::Reg(Reg::A),
                    mem,
                );
            }
            Instruction::LDH1(o) => {
                println!(
                    "Write to hram {:04x}, ({:02x}) <- {:02x}",
                    self.reg.pc, o, self.reg.a
                );
                if o == 0x85 {
                    // println!("Write to 85 {:04x} {:02x}", self.reg.pc, self.reg.a)
                    // self.breakpoint_delay = 10;
                }
                mem[0xFF00 | o as u16] = self.reg.a;
            }
            Instruction::LDH2(o) => {
                self.reg.a = mem[0xFF00 | o as u16];
                if !(o == 0x85 && self.reg.a == 0x00) {
                    println!(
                        "Read from hram {:04x}, ({:02x}): {:02x}",
                        self.reg.pc, o, self.reg.a
                    );
                    // self.breakpoint_delay = 190;
                }
            }
            Instruction::LDI(a, b) => {
                self.ld_8_bit(a, b, mem);
                self.reg
                    .set_pair(AddrReg::HL, self.reg.get_pair(AddrReg::HL).wrapping_add(1));
            }
            Instruction::LDD(a, b) => {
                self.ld_8_bit(a, b, mem);
                self.reg
                    .set_pair(AddrReg::HL, self.reg.get_pair(AddrReg::HL).wrapping_sub(1));
            }
            Instruction::LD16(reg, v) => {
                self.reg.set_pair(reg, v);
            }
            Instruction::LDSPHL => {
                self.reg
                    .set_pair(AddrReg::SP, self.reg.get_pair(AddrReg::HL));
            }
            Instruction::LDHL(offset) => {
                let hi = (self.reg.sp >> 8) as u8;
                let lo = (self.reg.sp & 0x00FF) as u8;
                let s = ((offset as u8) & MSB_MASK) != 0;

                let (r_lo, c) = lo.overflowing_add(offset as u8);
                let (_, h) = (lo << 4).overflowing_add((offset as u8) << 4);

                let r_hi = if c && !s {
                    hi.wrapping_add(1)
                } else if s && !c {
                    hi.wrapping_sub(1)
                } else {
                    hi
                };
                let r = ((r_hi as u16) << 8) | (r_lo as u16);
                self.reg.set_pair(AddrReg::HL, r);
                self.reg.set_flag(7, false);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, h);
                self.reg.set_flag(4, c);
            }
            Instruction::LDnn(r) => {
                mem[r] = (self.reg.sp & 0x00FF) as u8;
                mem[r + 1] = (self.reg.sp >> 8) as u8;
            }
            Instruction::PUSH(r) => {
                self.push(self.reg.get_pair(r), mem);
            }
            Instruction::POP(r) => {
                let val = self.pop(mem);
                self.reg.set_pair(r, val);
            }
            Instruction::ADD(l) => {
                self.reg.a = self.add_set_flags(l, false, mem);
            }
            Instruction::ADC(l) => {
                self.reg.a = self.add_set_flags(l, true, mem);
            }
            Instruction::ADD16(r) => {
                let hl = self.reg.get_pair(AddrReg::HL);
                let rhs = self.reg.get_pair(r);
                let (_, h) = (hl << 4).overflowing_add(rhs << 4);
                let (r, c) = hl.overflowing_add(rhs);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, h);
                self.reg.set_flag(4, c);
                self.reg.set_pair(AddrReg::HL, r);
            }
            Instruction::ADD16n(n) => {
                let hi = (self.reg.sp >> 8) as u8;
                let lo = (self.reg.sp & 0x00FF) as u8;
                let s = ((n as u8) & MSB_MASK) != 0;

                let (r_lo, c) = lo.overflowing_add(n as u8);
                let (_, h) = (lo << 4).overflowing_add((n as u8) << 4);

                let r_hi = if c && !s {
                    hi.wrapping_add(1)
                } else if s && !c {
                    hi.wrapping_sub(1)
                } else {
                    hi
                };
                let r = ((r_hi as u16) << 8) | (r_lo as u16);
                self.reg.set_flag(7, false);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, h);
                self.reg.set_flag(4, c);
                self.reg.set_pair(AddrReg::SP, r);
            }
            Instruction::SUB(l) => {
                self.reg.a = self.sub_set_flags(l, false, mem);
            }
            Instruction::SBC(l) => {
                self.reg.a = self.sub_set_flags(l, true, mem);
            }
            Instruction::AND(l) => {
                self.reg.a = self.and_set_flags(l, mem);
            }
            Instruction::OR(l) => {
                self.reg.a = self.or_set_flags(l, mem);
            }
            Instruction::XOR(l) => {
                self.reg.a = self.xor_set_flags(l, mem);
            }
            Instruction::SWAP(r) => {
                self.apply_to_7_bit_reg(Self::swap, r, mem);
                // let n = match l {
                //     DataLoc::Reg(r) => self.reg.get(r),
                //     DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
                //     _ => self.cpu_crash("Not in instruction set.".to_string()),
                // };
                // let n_new = Self::swap(n);
                // match l {
                //     DataLoc::Reg(r) => self.reg.set(r, n_new),
                //     DataLoc::AddrReg(AddrReg::HL) => {
                //         mem[self.reg.get_pair(AddrReg::HL)] = n_new;
                //     }
                //     _ => self.cpu_crash("Not in instruction set.".to_string()),
                // }
            }
            Instruction::CP(l) => {
                if matches!(l, DataLoc::Value(0xCB)) {
                    println!(
                        "TEST FAILED {:02x} {:02x} {:02x}",
                        mem[0xdef8], mem[0xdef9], mem[0xdefa]
                    );
                }
                let _ = self.sub_set_flags(l, false, mem);
            }
            Instruction::CALL(addr) => {
                self.call(addr, mem);
            }
            Instruction::CALLc(cond, addr) => {
                if self.reg.eval_condition(cond) {
                    conditional_extra_cycles = 3;
                    self.call(addr, mem);
                }
            }
            Instruction::RET => {
                self.ret(mem);
            }
            Instruction::RETc(cond) => {
                if self.reg.eval_condition(cond) {
                    conditional_extra_cycles = 3;
                    self.ret(mem);
                }
            }
            Instruction::RETI => {
                self.ret(mem);
                self.ime = true;
                self.instructions_out_of_interrupt = 0;
                println!("IME enabled");
                // self.cpu_crash("test".to_string());
            }
            Instruction::BIT(b, n) => {
                if b > 7 {
                    self.cpu_crash(format!("Invalid RES bit: {b}"));
                }
                let mask = (1 << b) as u8;
                let z = match n {
                    DataLoc::Reg(r) => self.reg.get(r) & mask,
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        mem[addr] & mask
                    }
                    _ => self.cpu_crash("Not in instruction set.".to_string()),
                };
                self.reg.set_flag(7, z == 0);
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, true);
            }
            Instruction::RES(b, n) => {
                if b > 7 {
                    self.cpu_crash(format!("Invalid RES bit: {b}"));
                }
                let mask = !((1 << b) as u8);
                match n {
                    DataLoc::Reg(r) => self.reg.set(r, self.reg.get(r) & mask),
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        mem[addr] &= mask;
                    }
                    _ => self.cpu_crash("Not in instruction set.".to_string()),
                }
            }
            Instruction::SET(b, n) => {
                if b > 7 {
                    panic!("Invalid SET bit: {b}");
                }
                let mask = (1 << b) as u8;
                match n {
                    DataLoc::Reg(r) => self.reg.set(r, self.reg.get(r) | mask),
                    DataLoc::AddrReg(AddrReg::HL) => {
                        let addr = self.reg.get_pair(AddrReg::HL);
                        mem[addr] |= mask;
                    }
                    _ => self.cpu_crash("Not in instruction set.".to_string()),
                }
            }
            Instruction::DAA => {
                // http://www.z80.info/z80syntx.htm#DAA
                let c = self.reg.get_flag(4);
                let h = self.reg.get_flag(5);
                let n = self.reg.get_flag(6);
                let a = self.reg.a;
                let lo = a & 0xF;

                let mut offset = 0u8;
                let mut c_after = false;
                if (!n && lo > 0x09) || h {
                    offset |= 0x06;
                }

                if (!n && a > 0x99) || c {
                    offset |= 0x60;
                    c_after = true;
                }

                self.reg.a = if !n {
                    a.wrapping_add(offset)
                } else {
                    a.wrapping_sub(offset)
                };

                self.reg.set_flag(7, self.reg.a == 0);
                self.reg.set_flag(5, false);
                self.reg.set_flag(4, c_after);
            }
            Instruction::CPL => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(6, true);
                self.reg.set_flag(5, true);
            }
            Instruction::CCF => {
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, false);
                self.reg.set_flag(4, !self.reg.get_flag(4));
            }
            Instruction::SCF => {
                self.reg.set_flag(6, false);
                self.reg.set_flag(5, false);
                self.reg.set_flag(4, true);
            }
            Instruction::NOP => {}
            Instruction::HALT => {
                if !self.ime && mem[0xFF0F] & mem[0xFFFF] != 0 {
                    self.cpu_crash("HALT BUG".to_string());
                }
                self.halted = true;
            }
            Instruction::STOP => {
                let button_pressed = mem[0xFF00] & 0x0F == 0;
                let interrupt_pending = mem[0xFF0F] & mem[0xFFFF] > 0;
                let speed_key_requested: bool = false;
                if button_pressed {
                    if interrupt_pending {
                        // 1 byte OP code
                    } else {
                        // self.next_byte(mem);
                        self.halted = true;
                    }
                } else {
                    if speed_key_requested {
                        if interrupt_pending {
                            if self.ime {
                                println!("Speed change");
                            } else {
                                self.cpu_crash("CPU glitch".to_string());
                            }
                        } else {
                            // self.next_byte(mem);
                            mem[0xFF04] = 0x00;
                            self.halted = true;
                            println!("Speed change");
                        }
                    } else {
                        if interrupt_pending {
                            mem[0xFF04] = 0x00;
                            println!("STOP MODE");
                        } else {
                            mem[0xFF04] = 0x00;
                            // self.next_byte(mem);
                            println!("STOP MODE");
                        }
                    }
                }
            }
            Instruction::DI => {
                println!("IME disabled");
                self.ime = false
            }
            Instruction::EI => self.ie_delay = 1,
            Instruction::RLCA => {
                self.reg.a = Self::rotate_left_into_carry(self.reg.a, false, &mut self.reg);
            }
            Instruction::RLA => {
                self.reg.a = Self::rotate_left_through_carry(self.reg.a, false, &mut self.reg);
            }
            Instruction::RRCA => {
                self.reg.a = Self::rotate_right_into_carry(self.reg.a, false, &mut self.reg);
            }
            Instruction::RRA => {
                self.reg.a = Self::rotate_right_through_carry(self.reg.a, false, &mut self.reg);
            }
            Instruction::RLC(r) => {
                self.apply_to_7_bit_reg(Self::rotate_left_into_carry, r, mem);
            }
            Instruction::RL(r) => {
                self.apply_to_7_bit_reg(Self::rotate_left_through_carry, r, mem);
            }
            Instruction::RRC(r) => {
                self.apply_to_7_bit_reg(Self::rotate_right_into_carry, r, mem);
            }
            Instruction::RR(r) => {
                self.apply_to_7_bit_reg(Self::rotate_right_through_carry, r, mem);
            }
            Instruction::SLA(r) => {
                self.apply_to_7_bit_reg(Self::shift_left_into_carry, r, mem);
            }
            Instruction::SRA(r) => {
                self.apply_to_7_bit_reg(Self::shift_right_into_carry_keep_msb, r, mem);
            }
            Instruction::SRL(r) => {
                self.apply_to_7_bit_reg(Self::shift_right_into_carry, r, mem);
            }
            Instruction::RST(proc) => {
                if proc == 0x38 {
                    // self.cpu_crash("HIT RST 0x38".to_string());
                }
                let curr = self.reg.pc; // PC was incremented, decrement to get current
                self.push(curr, mem);
                self.reg.pc = 0x0000 | proc as u16;
            }
        }

        if self.ie_delay == 0 {
            self.ime = true;
            println!("IME enabled");
        }
        if self.ie_delay >= 0 {
            self.ie_delay -= 1;
        }

        if self.breakpoint_delay > 0 {
            println!("TIMA: {:02x} {:04x}", mem[0xFF05], self.reg.pc);
            self.breakpoint_delay -= 1;
            if self.breakpoint_delay == 0 {
                self.cpu_crash("Breakpoint".to_string());
            }
        }

        let m_cycles = instruction.machine_cycles() as u32 + conditional_extra_cycles;
        self.update_timer(mem, m_cycles);

        // Debug log
        #[cfg(debug_assertions)]
        {
            let entry = ExecLog {
                pc: dbg_current_pc,
                byte,
                instruction: instruction.clone(),
                registers: self.reg.clone(),
            };
            // println!("Executed {:04x} {:02x} {:x?} \t\t {:x?}", entry.pc, entry.byte, entry.instruction, entry.registers);
            self.dbg_exec_log.push_front(entry);
            if self.dbg_exec_log.len() > 200 {
                let _ = self.dbg_exec_log.pop_back();
            }
        }

        if self.ime {
            self.instructions_out_of_interrupt += 1;
        }

        self.last = instruction;
        m_cycles
    }

    fn update_timer(&mut self, mem: &mut Memory, value: u32) {
        // TIMA register
        // TODO 4 is placeholder since any cpu instruction takes at least 4 cycles
        let mut tima_old = mem[0xFF05];
        let tma = mem[0xFF06];
        let tac = mem[0xFF07];
        let tac_clock_select = tac & 0b11;
        let tac_enable = (tac >> 2) & 1 != 0;

        if tac_enable {
            self.m_cycle_counter += value;
            // TODO I know there's a nice pattern with big magic here, but I can't figure it out.
            let inc_per_cycles = match tac_clock_select {
                0b00 => 256,
                0b01 => 4,
                0b10 => 16,
                0b11 => 64,
                _ => unreachable!(),
            };
            let mut tima_new = tima_old;
            while self.m_cycle_counter >= inc_per_cycles {
                self.m_cycle_counter -= inc_per_cycles;
                tima_new = tima_old.wrapping_add(1);
                if tima_new < tima_old {
                    tima_new = tma;
                    // Timer interrupt
                    mem[0xFF0F] |= 1 << 2;
                }
                tima_old = tima_new;
            }
            mem[0xFF05] = tima_new;
        }
    }

    fn apply_to_7_bit_reg<F>(&mut self, f: F, r: DataLoc, mem: &mut Memory)
    where
        F: Fn(u8, bool, &mut Registers) -> u8,
    {
        let n = match r {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        };
        let n_new = f(n, true, &mut self.reg);
        match r {
            DataLoc::Reg(r) => self.reg.set(r, n_new),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)] = n_new,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        };
    }

    fn swap(n: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let msn = n >> 4;
        let lsn = n & 0xF;
        let n_new = (lsn << 4) | msn;
        reg.set_flag(7, n_new == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, false);
        n_new
    }

    fn rotate_left_into_carry(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = a_old.rotate_left(1);
        // TODO gameboy manual says to set Z, z80 manual says not to. Not setting it gets
        //  test further with the same amount of instructions
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & MSB_MASK) != 0);
        a
    }

    fn rotate_left_through_carry(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = (a_old << 1) | reg.get_flag(4) as u8;
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & MSB_MASK) != 0);
        a
    }

    fn rotate_right_into_carry(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = a_old.rotate_right(1);
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & LSB_MASK) != 0);
        a
    }

    fn rotate_right_through_carry(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = (a_old >> 1) | ((reg.get_flag(4) as u8) << 7);
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & LSB_MASK) != 0);
        a
    }

    fn shift_left_into_carry(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = a_old << 1;
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & MSB_MASK) != 0);
        a
    }

    fn shift_right_into_carry(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = a_old >> 1;
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & LSB_MASK) != 0);
        a
    }

    fn shift_right_into_carry_keep_msb(a_old: u8, set_z: bool, reg: &mut Registers) -> u8 {
        let a = (a_old >> 1) | (a_old & MSB_MASK);
        reg.set_flag(7, a == 0 && set_z);
        reg.set_flag(6, false);
        reg.set_flag(5, false);
        reg.set_flag(4, (a_old & LSB_MASK) != 0);
        a
    }

    fn cpu_crash(&mut self, message: String) -> ! {
        self.print_exec_log();
        panic!("{}", message);
    }

    pub fn print_exec_log(&mut self) {
        while let Some(entry) = self.dbg_exec_log.pop_back() {
            println!(
                "Executed {:04x} {:02x} {:<30} {:x?}",
                entry.pc,
                entry.byte,
                format!("{:x?}", entry.instruction),
                entry.registers,
            );
        }
    }

    fn next_addr_lsb_first(&mut self, mem: &mut Memory) -> u16 {
        (self.next_byte(mem) as u16) | ((self.next_byte(mem) as u16) << 8)
    }

    fn push(&mut self, val: u16, mem: &mut Memory) {
        let ls = (val & 0xFF) as u8;
        let ms = (val >> 8) as u8;
        // mem[self.reg.sp] = ls;
        // mem[self.reg.sp - 1] = ms;
        mem[self.reg.sp - 1] = ms;
        mem[self.reg.sp - 2] = ls;
        self.reg.sp -= 2;
    }

    fn pop(&mut self, mem: &mut Memory) -> u16 {
        // let ls = mem[self.reg.sp + 2];
        // let ms = mem[self.reg.sp + 1];
        let ls = mem[self.reg.sp];
        let ms = mem[self.reg.sp + 1];
        self.reg.sp += 2;
        ((ms as u16) << 8) | (ls as u16)
    }

    fn call(&mut self, addr: u16, mem: &mut Memory) {
        let pc_next = self.reg.pc; // Program Counter is already at next instruction.
        self.push(pc_next, mem);
        self.reg.pc = addr;
    }

    fn ret(&mut self, mem: &mut Memory) {
        self.reg.pc = self.pop(mem);
    }

    fn add_set_flags(&mut self, l: DataLoc, add_carry: bool, mem: &mut Memory) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        } as u16;
        let a = self.reg.a as u16;
        let c_extra = add_carry as u16 * self.reg.get_flag(4) as u16;
        let r = a + n + c_extra;
        let h = (a & 0x0F) + (n & 0x0F) + c_extra > 0x0F;
        let c = (r >> 8) != 0;
        self.reg.set_flag(7, (r as u8) == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, h);
        self.reg.set_flag(4, c);
        r as u8
    }

    fn sub_set_flags(&mut self, l: DataLoc, add_carry: bool, mem: &mut Memory) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        };
        let a = self.reg.a;
        let c_extra = add_carry as u8 & self.reg.get_flag(4) as u8;

        let (r1, c1) = a.overflowing_sub(n);
        let (r, c2) = r1.overflowing_sub(c_extra);
        let c = c1 || c2;

        let (r1, h1) = (a << 4).overflowing_sub(n << 4);
        let (_, h2) = r1.overflowing_sub(c_extra << 4);
        let h = h1 || h2;

        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, true);
        self.reg.set_flag(5, h);
        self.reg.set_flag(4, c);

        // println!("Performed subtraction ({} {}) {} - {} -> {} ({} {})", add_carry, self.reg.get_flag(4), a, n, r as u8, h, c);
        (r & 0x00FF) as u8
    }

    fn and_set_flags(&mut self, l: DataLoc, mem: &mut Memory) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        };
        let r = self.reg.a & n;
        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, true);
        self.reg.set_flag(4, false);
        r
    }

    fn or_set_flags(&mut self, l: DataLoc, mem: &mut Memory) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        };
        let r = self.reg.a | n;
        self.reg.set_flag(7, r == 0);
        self.reg.set_flag(6, false);
        self.reg.set_flag(5, false);
        self.reg.set_flag(4, false);
        r
    }

    fn xor_set_flags(&mut self, l: DataLoc, mem: &mut Memory) -> u8 {
        let n = match l {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(AddrReg::HL) => mem[self.reg.get_pair(AddrReg::HL)],
            DataLoc::Value(v) => v,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
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

    fn ld_8_bit(&mut self, to: DataLoc, from: DataLoc, mem: &mut Memory) {
        // TODO this might be optimised by making the datalocs generic
        let v = match from {
            DataLoc::Reg(r) => self.reg.get(r),
            DataLoc::AddrReg(r) => mem[self.reg.get_pair(r)],
            DataLoc::Addr(addr) => mem[addr],
            DataLoc::Value(v) => v,
        };
        match to {
            DataLoc::Reg(r) => self.reg.set(r, v),
            DataLoc::AddrReg(r) => mem[self.reg.get_pair(r)] = v,
            DataLoc::Addr(addr) => mem[addr] = v,
            _ => self.cpu_crash("Not in instruction set.".to_string()),
        }
    }
}
