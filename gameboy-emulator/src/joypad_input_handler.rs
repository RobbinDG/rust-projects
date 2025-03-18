use std::sync::{Arc, Mutex};
use minifb::InputCallback;
use crate::joypad::JoyPad;

pub struct JoypadInputHandler {
    joy_pad: Arc<Mutex<JoyPad>>
}

impl JoypadInputHandler {
    pub fn new(joy_pad: Arc<Mutex<JoyPad>>) -> Self {
        Self { joy_pad }
    }
}

impl InputCallback for JoypadInputHandler {
    fn add_char(&mut self, uni_char: u32) {
        let c = std::char::from_u32(uni_char).unwrap();
        let mut jp = self.joy_pad.lock().unwrap();
        jp.reset();
        match c {
            'w' => jp.up = true,
            'a' => jp.left = true,
            's' => jp.down = true,
            'd' => jp.right = true,
            'j' => jp.a = true,
            'i' => jp.b = true,
            'n' => jp.start = true,
            'm' => jp.select = true,
            _ => {}
        }
    }
}