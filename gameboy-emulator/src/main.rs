use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};
use minifb::{Key, Window, WindowOptions};
use cartridge_header::CartridgeHeader;
use cpu::CPU;
mod cartridge_header;
mod cpu;
mod memory;
mod instruction;
mod dataloc;
mod addrreg;
mod reg;
mod condition;
mod register;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const REG_INTERRUPT_FLAG: u16 = 0xFF0F;
const REG_INTERRUPT_ENABLE: u16 = 0xFFFF;

const MSB_MASK: u8 = 0b10000000;
const LSB_MASK: u8 = 0b00000001;
const LS_BYTE_MASK: u16 = 0x00FF;
const MS_BYTE_MASK: u16 = 0xFF00;

struct GameBoy {
    rom: Vec<u8>,
    cpu: CPU,
}

impl GameBoy {
    fn from_cartridge(cartridge_filename: &'static str) -> Self {
        let mut f = File::open(&cartridge_filename).expect("no file found");
        let metadata = fs::metadata(&cartridge_filename).expect("unable to read metadata");
        let mut rom = vec![0; metadata.len() as usize];
        f.read(&mut rom).expect("buffer overflow");

        // Read cartridge header
        let header = CartridgeHeader::read(&rom);
        println!("{:x?}", header);
        // Read ROM and RAM sizes on cartridge

        let mut cpu = CPU::new(rom.clone());
        Self { rom, cpu }
    }

    pub fn start(&mut self) {
        for _ in 0usize..1000 {
            self.cpu.step();
        }

        let mut window = Window::new(
        "Pixel Grid - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap();

        let mut buffer = vec![0u32; WIDTH * HEIGHT];

        while window.is_open() && !window.is_key_down(Key::Escape) {
            for i in buffer.iter_mut() {
                *i = 0x00FF00; // Green pixels
            }

            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    }
}

fn main() {
    let filename = "./Pokemon Red (UE) [S][!].gb";
    let mut gb = GameBoy::from_cartridge(filename);
    gb.start();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inc() {
        let rom = vec![0x3E, 0b1110, 0x3C, 0x3C, 0x3C];
        let mut cpu = CPU::new(rom);
        cpu.reg.pc = 0;

        assert_eq!(cpu.reg.a, 0);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b1110);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b1111);
        assert_eq!(cpu.reg.get_flag(7), false);
        assert_eq!(cpu.reg.get_flag(6), false);
        assert_eq!(cpu.reg.get_flag(5), false);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b10000);
        assert_eq!(cpu.reg.get_flag(7), false);
        assert_eq!(cpu.reg.get_flag(6), false);
        assert_eq!(cpu.reg.get_flag(5), true);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b10001);
        assert_eq!(cpu.reg.get_flag(7), false);
        assert_eq!(cpu.reg.get_flag(6), false);
        assert_eq!(cpu.reg.get_flag(5), false);
    }

    #[test]
    fn test_sub_eq() {
        let rom = vec![0x3E, 0b1110, 0x97];
        let mut cpu = CPU::new(rom);
        cpu.reg.pc = 0;

        assert_eq!(cpu.reg.a, 0);
        cpu.step();
        assert_eq!(cpu.reg.a, 0b1110);
        cpu.step();

        assert_eq!(cpu.reg.a, 0);
        assert_eq!(cpu.reg.get_flag(7), true);
        assert_eq!(cpu.reg.get_flag(6), true);
        assert_eq!(cpu.reg.get_flag(5), true);
        assert_eq!(cpu.reg.get_flag(4), true);
    }
}
