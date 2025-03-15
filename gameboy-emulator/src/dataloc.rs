use crate::reg::Reg;
use crate::addrreg::AddrReg;

#[derive(Debug, Clone)]
pub enum DataLoc {
    Reg(Reg),
    AddrReg(AddrReg),
    Addr(u16),
    Value(u8),
}