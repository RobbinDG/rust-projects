use crate::memory::Memory;
use crate::ppu::PPU;
use cartridge_header::CartridgeHeader;
use cpu::CPU;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};

mod addrreg;
mod cartridge_header;
mod condition;
mod cpu;
mod dataloc;
mod instruction;
mod memory;
mod ppu;
mod reg;
mod register;

const REG_INTERRUPT_FLAG: u16 = 0xFF0F;
const REG_INTERRUPT_ENABLE: u16 = 0xFFFF;

const MSB_MASK: u8 = 0b10000000;
const LSB_MASK: u8 = 0b00000001;
const LS_BYTE_MASK: u16 = 0x00FF;
const MS_BYTE_MASK: u16 = 0xFF00;

struct GameBoy {
    mem: Memory,
    cpu: CPU,
    ppu: PPU,
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

        let mem = Memory::new(rom);
        let cpu = CPU::new();
        let ppu = PPU::new();
        Self { mem, cpu, ppu }
    }

    pub fn start(mut self) {
        for _ in 0usize..150000 {
            self.mem = self.cpu.run_cycle(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
        }
        self.cpu.print_exec_log()
    }
}

fn main() {
    let filename = "./Pokemon Red (UE) [S][!].gb";
    let mut gb = GameBoy::from_cartridge(filename);
    gb.start();
}
