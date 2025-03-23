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
    JPc(Condition, u16),
    CALL(u16),
    CALLc(Condition, u16),
    RST(u8),
    RET,
    RETc(Condition),
    RETI,
}

impl Instruction {
    // pub fn clock_cycles(&self) -> usize {
    //     match self {
    //         Instruction::LD(_, _) = {}
    //         Instruction::LD5 => {}
    //         Instruction::LD6 => {}
    //         Instruction::LDH1(_) => {}
    //         Instruction::LDH2(_) => {}
    //         Instruction::LD16(_, _) => {}
    //         Instruction::LDSPHL => {}
    //         Instruction::LDHL(_) => {}
    //         Instruction::LDnn(_) => {}
    //         Instruction::PUSH(_) => {}
    //         Instruction::POP(_) => {}
    //         Instruction::ADD(_) => {}
    //         Instruction::ADC(_) => {}
    //         Instruction::SUB(_) => {}
    //         Instruction::SBC(_) => {}
    //         Instruction::AND(_) => {}
    //         Instruction::OR(_) => {}
    //         Instruction::XOR(_) => {}
    //         Instruction::CP(_) => {}
    //         Instruction::INC(_) => {}
    //         Instruction::DEC(_) => {}
    //         Instruction::ADD16(_) => {}
    //         Instruction::ADD16n(_) => {}
    //         Instruction::INC16(_) => {}
    //         Instruction::DEC16(_) => {}
    //         Instruction::SWAP(_) => {}
    //         Instruction::DAA => {}
    //         Instruction::CPL => {}
    //         Instruction::CCF => {}
    //         Instruction::SCF => {}
    //         Instruction::NOP => {}
    //         Instruction::HALT => {}
    //         Instruction::STOP => {}
    //         Instruction::DI => {}
    //         Instruction::EI => {}
    //         Instruction::RLCA => {}
    //         Instruction::RLA => {}
    //         Instruction::RRCA => {}
    //         Instruction::RRA => {}
    //         Instruction::RLC(_) => {}
    //         Instruction::RL(_) => {}
    //         Instruction::RRC(_) => {}
    //         Instruction::RR(_) => {}
    //         Instruction::SLA(_) => {}
    //         Instruction::SRA(_) => {}
    //         Instruction::SRL(_) => {}
    //         Instruction::BIT(_, _) => {}
    //         Instruction::SET(_, _) => {}
    //         Instruction::RES(_, _) => {}
    //         Instruction::JP1(_) => {}
    //         Instruction::JP2(_, _) => {}
    //         Instruction::JP3 => {}
    //         Instruction::JR4(_) => {}
    //         Instruction::JR5(_, _) => {}
    //         Instruction::JPc(_, _) => {}
    //         Instruction::CALL(_) => {}
    //         Instruction::CALLc(_, _) => {}
    //         Instruction::RST(_) => {}
    //         Instruction::RET => {}
    //         Instruction::RETc(_) => {}
    //         Instruction::RETI => {}
    //     }
    // }
}