use crate::condition::Condition;
use crate::addrreg::AddrReg;
use crate::dataloc::DataLoc;

#[derive(Debug, Clone)]
pub enum Instruction {
    INC(DataLoc),
    INC16(AddrReg),
    DEC(DataLoc),
    DEC16(AddrReg),
    LD1(DataLoc, u8),
    LD2(DataLoc, DataLoc),
    LD3(DataLoc),
    LD4(DataLoc),
    LD5,
    LD6,
    LDH1(u8),
    LDH2(u8),
    LDI1,
    LDI2,
    LDD1,
    LDD2,
    LD16(AddrReg, u16),
    LDSPHL,
    LDHL(i8),
    LDnn(u16),
    PUSH(AddrReg),
    POP(AddrReg),
    ADD(DataLoc),
    ADD16(AddrReg),
    ADD16n(i8),
    SUB(DataLoc),
    ADC(DataLoc),
    SBC(DataLoc),
    NEG,
    AND(DataLoc),
    OR(DataLoc),
    XOR(DataLoc),
    CP(DataLoc),
    JP1(u16),
    JP2(Condition, u16),
    JP3,
    JR4(i8),
    JR5(Condition, i8),
    JPc(Condition, u16),
    CALL(u16),
    CALLc(Condition, u16),
    RET,
    RETc(Condition),
    RETI,
    BIT(u8, DataLoc),
    RES(u8, DataLoc),
    SET(u8, DataLoc),
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
    RST(u8),
}