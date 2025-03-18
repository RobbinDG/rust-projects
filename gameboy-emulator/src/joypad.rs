use crate::memory::Memory;

pub struct JoyPad {
    previous: u8,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl JoyPad {
    pub fn new() -> Self {
        JoyPad {
            previous: 0x0F,
            start: false,
            select: false,
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }

    pub fn reset(&mut self) {
        self.start = false;
        self.select = false;
        self.a = false;
        self.b = false;
        self.up = false;
        self.down = false;
        self.left = false;
        self.right = false;
    }

    pub fn update(&mut self, mem: &mut Memory) {
        mem[0xFF00] |= 0x0F;
        if mem[0xFF00] >> 4 == 0b10 {
            if self.start {
                self.set_button_bit(mem, 0);
            }
            if self.select {
                self.set_button_bit(mem, 1);
            }
            if self.a {
                self.set_button_bit(mem, 2);
            }
            if self.b {
                self.set_button_bit(mem, 3);
            }
        }
        if mem[0xFF00] >> 4 == 0b01 {
            if self.down {
                self.set_button_bit(mem, 0);
            }
            if self.up {
                self.set_button_bit(mem, 1);
            }
            if self.left {
                self.set_button_bit(mem, 2);
            }
            if self.right {
                self.set_button_bit(mem, 3);
            }
        }
        self.previous = mem[0xFF00];
    }

    pub fn set_button_bit(&mut self, mem: &mut Memory, button: u8) {
        if button > 3 {
            panic!("Button out of range");
        }
        if self.previous & (1 << button) != 0 {
            mem[0xFF0F] |= 1 << 4;
        }
        mem[0xFF00] &= 0x10 | !(1 << button);
    }
}
