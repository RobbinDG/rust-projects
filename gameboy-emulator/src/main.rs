use crate::joypad::JoyPad;
use crate::memory::Memory;
use crate::ppu::PPU;
use cartridge_header::CartridgeHeader;
use cpu::CPU;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::ops::{Index, IndexMut};
use std::thread::sleep;
use std::time::Duration;

mod addrreg;
mod cartridge_header;
mod condition;
mod cpu;
mod dataloc;
mod instruction;
mod joypad;
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
    joy_pad: JoyPad,
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

        let jp = JoyPad::new();
        let mem = Memory::new(rom);
        let cpu = CPU::new();
        let ppu = PPU::new();
        Self {
            mem,
            cpu,
            ppu,
            joy_pad: jp,
        }
    }

    pub fn start(mut self) {
        for _ in 0usize..1_100_000 {
            // DIV register
            self.mem[0xFF04] = self.mem[0xFF04].wrapping_add(1);
            // TIMA register
            // TODO 4 is placeholder since any cpu instruction takes at least 4 cycles
            let tima_old = self.mem[0xFF05];
            // Check TAC enable TODO TAC clock select
            if (self.mem[0xFF07] >> 2) & 1 != 0 {
                self.mem[0xFF05] = self.mem[0xFF05].wrapping_add(4);
                if self.mem[0xFF05] < tima_old {
                    self.mem[0xFF05] = self.mem[0xFF06];
                    // Timer interrupt
                    if (self.mem[0xFFFF] >> 2) & 1 != 0 {
                        self.mem[0xFF0F] |= 1 << 2;
                    }
                }
            }

            self.joy_pad.update(&mut self.mem);
            self.cpu.check_interrupts(&mut self.mem);
            self.mem = self.cpu.run_cycle(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
        }
        self.cpu.print_exec_log();
        BufWriter::new(File::create("./tile_ram.bin").unwrap())
            .write_all(&self.mem.tile_ram)
            .unwrap();
        BufWriter::new(File::create("./background_map.bin").unwrap())
            .write_all(&self.mem.background_map)
            .unwrap();
        BufWriter::new(File::create("./sprite.bin").unwrap())
            .write_all(&self.mem.sprite)
            .unwrap();
        loop {
            sleep(Duration::from_millis(1000));
        }
    }
}

fn main() {
    let filename = "./Pokemon Red (UE) [S][!].gb";
    let mut gb = GameBoy::from_cartridge(filename);
    gb.start();
}
