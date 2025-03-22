use crate::joypad::JoyPad;
use crate::joypad_input_handler::JoypadInputHandler;
use crate::memory::Memory;
use crate::ppu::PPU;
use cartridge_header::CartridgeHeader;
use cpu::CPU;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

mod addrreg;
mod cartridge_header;
mod condition;
mod cpu;
mod dataloc;
mod instruction;
mod joypad;
mod joypad_input_handler;
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
    joy_pad: Arc<Mutex<JoyPad>>,
}

impl GameBoy {
    pub fn from_cartridge(cartridge_filename: &'static str) -> Self {
        let boot_rom = Self::read_bin_file(&"dmg_boot.bin");
        let rom = Self::read_bin_file(&cartridge_filename);

        // Read cartridge header
        let header = CartridgeHeader::read(&rom);
        println!("{:x?}", header);
        assert_ne!(header.cgb, 0xC0, "Not compatible with Monochrome");

        let jp = Arc::new(Mutex::new(JoyPad::new()));
        let mem = Memory::new(boot_rom, rom, header.memory_bank_controller().unwrap());
        let cpu = CPU::new();
        let ppu = PPU::new(JoypadInputHandler::new(jp.clone()));
        GameBoy {
            mem,
            cpu,
            ppu,
            joy_pad: jp,
        }
    }

    pub fn skip_boot_rom(&mut self) {
        self.cpu.reg.pc = 0x100;
        self.mem[0xFF50] = 0x01;
    }

    fn read_bin_file(cartridge_filename: &&str) -> Vec<u8> {
        let mut f = File::open(&cartridge_filename).expect("no file found");
        let metadata = fs::metadata(&cartridge_filename).expect("unable to read metadata");
        let mut rom = vec![0; metadata.len() as usize];
        f.read(&mut rom).expect("buffer overflow");
        rom
    }

    pub fn start(mut self) {
        for _ in 0usize..1000_400_000 {
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

            self.joy_pad.lock().unwrap().update(&mut self.mem);
            self.cpu.check_interrupts(&mut self.mem);
            self.mem = self.cpu.run_cycle(self.mem);
            self.mem = self.ppu.run_dot(self.mem);
            // self.mem = self.ppu.run_dot(self.mem);
            // self.mem = self.ppu.run_dot(self.mem);
            // self.mem = self.ppu.run_dot(self.mem);
        }
        self.cpu.print_exec_log();

        for i in 0..256 {
            if self.cpu.instructions_count[i] > 0 {
                println!("{:02x}: {}", i as u8, self.cpu.instructions_count[i]);
            }
        }
        for i in 256..(2 * 256) {
            if self.cpu.instructions_count[i] > 0 {
                println!(
                    "cb {:02x}: {}",
                    (i - 256) as u8,
                    self.cpu.instructions_count[i]
                );
            }
        }

        self.mem.write_contents();
        loop {
            sleep(Duration::from_millis(1000));
        }
    }
}

fn main() {
    // let filename = "./Pokemon Red (UE) [S][!].gb";
    // let filename = "./Tetris (JUE) (V1.1) [!].gb";
    let filename = "./cpu_instrs.gb";
    let mut gb = GameBoy::from_cartridge(filename);
    gb.skip_boot_rom();
    gb.start();
}
