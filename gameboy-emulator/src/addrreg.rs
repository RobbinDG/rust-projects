#[derive(Debug, Copy, Clone)]
pub enum AddrReg {
    /// (BC)
    BC,
    /// (DE)
    DE,
    /// (HL)
    HL,
    SP,
    AF,
}