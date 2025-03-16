use crate::memory::Memory;

pub struct JoyPad {}

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad {}
    }

    pub fn update(&mut self, mem: &mut Memory) {
        if mem[0xFF00] >> 4 == 0x1 {
            mem[0xFF00] &= 0x10;
        } else {
            mem[0xFF00] |= 0x0F;
        }
    }
}
