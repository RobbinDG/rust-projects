use crate::condition::Condition;
use crate::addrreg::AddrReg;
use crate::dataloc::DataLoc;

#[derive(Debug, Clone)]
pub enum Instruction {
    LD(DataLoc, DataLoc),
    LD5,
    LD6,
    LDD(DataLoc, DataLoc),
    LDI(DataLoc, DataLoc),
    LDH1(u8),
    LDH2(u8),
    LD16(AddrReg, u16),
    LDSPHL,
    LDHL(i8),
    LDnn(u16),
    PUSH(AddrReg),
    POP(AddrReg),
    ADD(DataLoc),
    ADC(DataLoc),
    SUB(DataLoc),
    SBC(DataLoc),
    AND(DataLoc),
    OR(DataLoc),
    XOR(DataLoc),
    CP(DataLoc),
    INC(DataLoc),
    DEC(DataLoc),
    ADD16(AddrReg),
    ADD16n(i8),
    INC16(AddrReg),
    DEC16(AddrReg),
    SWAP(DataLoc),
    DAA,
    CPL,
    CCF,
    SCF,
    NOP,
    HALT,
    STOP,
    DI,
    EI,
    RLCA,
    RLA,
    RRCA,
    RRA,
    RLC(DataLoc),
    RL(DataLoc),
    RRC(DataLoc),
    RR(DataLoc),
    SLA(DataLoc),
    SRA(DataLoc),
    SRL(DataLoc),
    BIT(u8, DataLoc),
    SET(u8, DataLoc),
    RES(u8, DataLoc),
    JP1(u16),
    JP2(Condition, u16),
    JP3,
    JR4(i8),
    JR5(Condition, i8),
    CALL(u16),
    CALLc(Condition, u16),
    RST(u8),
    RET,
    RETc(Condition),
    RETI,
}

impl Instruction {
    pub fn clock_cycles(&self) -> u8 {
        match self {
            Instruction::LD(a, b) => match (a, b) {
                (DataLoc::Reg(_), DataLoc::Reg(_)) => 4,
                (DataLoc::Reg(_), DataLoc::AddrReg(_)) => 8,
                (DataLoc::Reg(_), DataLoc::Addr(_)) => 16,
                (DataLoc::Reg(_), DataLoc::Value(_)) => 8,
                (DataLoc::AddrReg(_), DataLoc::Reg(_)) => 8,
                (DataLoc::AddrReg(_), DataLoc::Value(_)) => 12,
                (DataLoc::Addr(_), DataLoc::Reg(_)) => 16,
                (a, b) => panic!("{:?} {:?}", a, b),
            },
            Instruction::LD5 => 8,
            Instruction::LD6 => 8,
            Instruction::LDD(_, _) => 8,
            Instruction::LDI(_, _) => 8,
            Instruction::LDH1(_) => 12,
            Instruction::LDH2(_) => 12,
            Instruction::LD16(_, _) => 12,
            Instruction::LDSPHL => 8,
            Instruction::LDHL(_) => 12,
            Instruction::LDnn(_) => 20,
            Instruction::PUSH(_) => 16,
            Instruction::POP(_) => 12,
            Instruction::ADD(v) => Self::simple_arith_clock_cycles(v),
            Instruction::ADC(v) => Self::simple_arith_clock_cycles(v),
            Instruction::SUB(v) => Self::simple_arith_clock_cycles(v),
            Instruction::SBC(v) => Self::simple_arith_clock_cycles(v),
            Instruction::AND(v) => Self::simple_arith_clock_cycles(v),
            Instruction::OR(v) => Self::simple_arith_clock_cycles(v),
            Instruction::XOR(v) => Self::simple_arith_clock_cycles(v),
            Instruction::CP(v) => Self::simple_arith_clock_cycles(v),
            Instruction::INC(v) => match v {
                DataLoc::Reg(_) => 4,
                DataLoc::AddrReg(_) => 12,
                _ => 4,
            },
            Instruction::DEC(v) => match v {
                DataLoc::Reg(_) => 4,
                DataLoc::AddrReg(_) => 12,
                _ => 4,
            },
            Instruction::ADD16(_) => 8,
            Instruction::ADD16n(_) => 16,
            Instruction::INC16(_) => 8,
            Instruction::DEC16(_) => 8,
            Instruction::SWAP(v) => match v {
                DataLoc::Reg(_) => 8,
                DataLoc::AddrReg(_) => 16,
                _ => 8,
            },
            Instruction::DAA => 4,
            Instruction::CPL => 4,
            Instruction::CCF => 4,
            Instruction::SCF => 4,
            Instruction::NOP => 4,
            Instruction::HALT => 4,
            Instruction::STOP => 4,
            Instruction::DI => 4,
            Instruction::EI => 4,
            Instruction::RLCA => 4,
            Instruction::RLA => 4,
            Instruction::RRCA => 4,
            Instruction::RRA => 4,
            Instruction::RLC(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::RL(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::RRC(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::RR(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::SLA(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::SRA(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::SRL(v) => Self::clock_cycles_rotate_shift(v),
            Instruction::BIT(_, v) => Self::clock_cycles_bit(v),
            Instruction::SET(_, v) => Self::clock_cycles_rotate_shift(v),
            Instruction::RES(_, v) => Self::clock_cycles_rotate_shift(v),
            Instruction::JP1(_) => 16,
            Instruction::JP2(_, _) => 12,
            Instruction::JP3 => 4,
            Instruction::JR4(_) => 12,
            Instruction::JR5(_, _) => 8,
            Instruction::CALL(_) => 24,
            Instruction::CALLc(_, _) => 12,
            Instruction::RST(_) => 16,
            Instruction::RET => 16,
            Instruction::RETc(_) => 8,
            Instruction::RETI => 16,
        }
    }

    fn clock_cycles_rotate_shift(v: &DataLoc) -> u8 {
        match v {
            DataLoc::Reg(_) => 8,
            DataLoc::AddrReg(_) => 16,
            _ => 8,
        }
    }

    fn clock_cycles_bit(v: &DataLoc) -> u8 {
        match v {
            DataLoc::Reg(_) => 8,
            DataLoc::AddrReg(_) => 12,
            _ => 8,
        }
    }

    fn simple_arith_clock_cycles(v: &DataLoc) -> u8 {
        match v {
            DataLoc::Reg(_) => 4,
            DataLoc::AddrReg(_) => 8,
            DataLoc::Value(_) => 8,
            _ => 4,
        }
    }

    pub fn machine_cycles(&self) -> u8 {
        self.clock_cycles() / 4
    }
}