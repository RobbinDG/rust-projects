use crate::memory::Memory;

pub struct JoyPad {}

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad {}
    }

    pub fn update(&mut self, mem: &mut Memory) {
        if mem[0xFF00] >> 4 == 0x1 {
            self.press_button(mem, 0);
            self.press_button(mem, 1);
            self.press_button(mem, 2);
            self.press_button(mem, 3);
        } else {
            mem[0xFF00] |= 0x0F;
        }
    }

    pub fn press_button(&mut self, mem: &mut Memory, button: u8) {
        if button > 3 {
            panic!("Button out of range");
        }
        if mem[0xFF00] & (1 << button) != 0 {
            mem[0xFF0F] |= 1 << 4;
        }
        mem[0xFF00] &= 0x10 | !(1 << button);
    }
}
