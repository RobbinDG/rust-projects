use crate::memory::Memory;
use crate::ppu::PPU;
use cartridge_header::CartridgeHeader;
use cpu::CPU;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
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
        for _ in 0usize..300000 {
            self.mem = self.cpu.run_cycle(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
        }
        self.cpu.print_exec_log();
        BufWriter::new(File::create("./tile_ram.bin").unwrap()).write_all(&self.mem.tile_ram).unwrap();
        BufWriter::new(File::create("./background_map.bin").unwrap()).write_all(&self.mem.background_map).unwrap();
        BufWriter::new(File::create("./sprite.bin").unwrap()).write_all(&self.mem.sprite).unwrap();
    }
}

fn main() {
    let filename = "./Pokemon Red (UE) [S][!].gb";
    let mut gb = GameBoy::from_cartridge(filename);
    gb.start();
}
