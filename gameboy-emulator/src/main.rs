use crate::joypad::JoyPad;
use crate::joypad_input_handler::JoypadInputHandler;
use crate::memory::Memory;
use crate::ppu::PPU;
use cartridge_header::CartridgeHeader;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamInstant};
use cpu::CPU;
use log::debug;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use crate::apu::APU;

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
mod apu;

const CLOCK_FREQ_UPDATE_INTERVAL: u32 = 1_000_000;

struct GameBoy {
    mem: Memory,
    cpu: CPU,
    ppu: PPU,
    apu: APU,
    apu_div: u8,
    joy_pad: Arc<Mutex<JoyPad>>,
    cpu_last_cycle_cnt_reset: SystemTime,
    cpu_cycle_counter: u32,
    div_counter: u8,
    dot_counter: u32,
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
        let mem = Memory::new(boot_rom, rom, header).unwrap();
        let cpu = CPU::new();
        let ppu = PPU::new(JoypadInputHandler::new(jp.clone()));
        let apu = APU::new();
        GameBoy {
            mem,
            cpu,
            ppu,
            apu,
            apu_div: 0,
            joy_pad: jp,
            cpu_last_cycle_cnt_reset: SystemTime::now(),
            cpu_cycle_counter: 0,
            div_counter: 0,
            dot_counter: 0,
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
        for _ in 0usize..100_000_000 {
            {
                self.joy_pad.lock().unwrap().update(&mut self.mem);
            }
            self.cpu.check_interrupts(&mut self.mem);
            let last_div_ctr = self.div_counter;
            let m_cycles = self.cpu.run_cycle(&mut self.mem);
            self.div_counter = self.div_counter.wrapping_add(4 * m_cycles as u8);
            if self.div_counter < last_div_ctr {
                // DIV register
                let last_div = self.mem[0xFF04];
                let new_div = last_div.wrapping_add(1);
                if new_div & 0b0000_1000 == 0 && last_div & 0b0000_1000 != 0 {
                    self.apu_div = self.apu_div.wrapping_add(1);
                }
                self.mem[0xFF04] = new_div;
            }
            for _ in 0..(m_cycles * 4) {
                self.ppu.run_dot(&mut self.mem);
                self.dot_counter = self.dot_counter.wrapping_add(1);
                if self.dot_counter % 4 == 0 {
                    self.apu.clock_pulse(&mut self.mem);
                }
            }
            self.cpu_cycle_counter += m_cycles;
            if self.cpu_cycle_counter >= CLOCK_FREQ_UPDATE_INTERVAL {
                self.cpu_cycle_counter -= CLOCK_FREQ_UPDATE_INTERVAL;
                let cycle_time = SystemTime::now();
                let dt = cycle_time
                    .duration_since(self.cpu_last_cycle_cnt_reset)
                    .unwrap();
                self.cpu_last_cycle_cnt_reset = cycle_time;
                debug!(
                    "Clock Freq: {} MHz",
                    CLOCK_FREQ_UPDATE_INTERVAL as f32 / dt.as_secs_f32() / 1_000_000.0
                );
            }
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

        self.mem.write_contents().unwrap();
        PPU::render_all_tiles(&self.mem);
        loop {
            sleep(Duration::from_millis(1000));
        }
    }
}

fn main() {
    // let filename = "./Pokemon Red (UE) [S][!].gb";
    // let filename = "./Tetris (JUE) (V1.1) [!].gb";
    // let filename = "./cpu_instrs.gb";
    // let filename = "./instr_timing.gb";
    let filename = "./dmg_sound.gb";
    let mut gb = GameBoy::from_cartridge(filename);
    gb.skip_boot_rom();
    gb.start();
}
