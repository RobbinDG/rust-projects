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