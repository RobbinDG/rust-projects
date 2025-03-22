use std::cmp::max;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::ops::{Index, IndexMut};

pub struct MBC3 {
    pub rom_bank_reg: u8,
}

impl MBC3{
    pub fn new() -> Self {
        Self { rom_bank_reg: 1 }
    }

    pub fn rom_read_addr(&self, addr: u16) -> u32 {
        if !matches!(addr, 0x2000..=0x3FFF) {
            panic!("Invalid write to memory bank: {:04x}", addr);
        }
        max(self.rom_bank_reg, 1) as u32 * 0x4000 + (addr as u32 - 0x4000)
    }

    pub fn rom_write(&mut self, addr: u16) -> &mut u8 {
        if !matches!(addr, 0x2000..=0x3FFF) {
            panic!("Invalid write to memory bank: {:04x}", addr);
        }
        &mut self.rom_bank_reg
    }
}

pub enum MemoryBankController {
    MBC3(MBC3),
}

impl MemoryBankController {
    pub fn rom_read_addr(&self, addr: u16) -> u32 {
        match self {
            MemoryBankController::MBC3(c) => c.rom_read_addr(addr),
        }
    }

    pub fn rom_write(&mut self, addr: u16) -> &mut u8 {
        match self {
            MemoryBankController::MBC3(c) => c.rom_write(addr),
        }
    }
}

pub struct Memory {
    bank_ctrl: MemoryBankController,
    boot_rom: Vec<u8>,
    rom: Vec<u8>,
    pub tile_ram: [u8; 0x1800],
    pub background_map: [u8; 0x0800],
    cartridge_ram: [u8; 0x2000],
    wram: [u8; 0x2000],
    pub sprite: [u8; 0xA0],
    io: [u8; 0x80],
    pub high_ram: [u8; 0x80],
    unused_response: u8,
    unused_write_dummy: u8,
}

impl Memory {
    pub fn new(boot_rom: Vec<u8>, rom: Vec<u8>, bank_ctrl: MemoryBankController) -> Self {
        Self {
            bank_ctrl,
            boot_rom,
            rom,
            tile_ram: [0; 0x1800],
            background_map: [0; 0x0800],
            cartridge_ram: [0; 0x2000],
            wram: [0; 0x2000],
            sprite: [0; 0xA0],
            io: [0; 0x80],
            high_ram: [0; 0x80],
            unused_response: 0xFF,
            unused_write_dummy: 0,
        }
    }

    pub fn write_contents(&self) -> io::Result<()> {
        BufWriter::new(File::create("./tile_ram.bin")?).write_all(&self.tile_ram)?;
        BufWriter::new(File::create("./background_map.bin")?).write_all(&self.background_map)?;
        BufWriter::new(File::create("./sprite.bin")?).write_all(&self.sprite)?;
        BufWriter::new(File::create("./high_ram.bin")?).write_all(&self.high_ram)?;
        Ok(())
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, addr: u16) -> &Self::Output {
        match addr {
            0x0000..=0x00FF => {
                if self[0xFF50] == 0 {
                    &self.boot_rom[addr as usize]
                } else {
                    &self.rom[addr as usize]
                }
            }
            0x0100..=0x3FFF => &self.rom[addr as usize],
            0x4000..=0x7FFF => &self.rom[self.bank_ctrl.rom_read_addr(addr) as usize],
            0x8000..=0x97FF => &self.tile_ram[(addr - 0x8000) as usize],
            0x9800..=0x9FFF => &self.background_map[(addr - 0x9800) as usize],
            0xA000..=0xBFFF => &self.cartridge_ram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => &self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => &self.wram[(addr - 0xE000) as usize], // Echo RAM
            0xFE00..=0xFE9F => &self.sprite[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => &self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFF => &self.high_ram[(addr - 0xFF80) as usize],
            _ => {
                println!("Unused memory {:04x}", addr);
                &self.unused_response
            }
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        match addr {
            0x0000..=0x7FFF => self.bank_ctrl.rom_write(addr),
            0x8000..=0x97FF => {
                // println!("write to tile ram {:04x}", addr);
                &mut self.tile_ram[(addr - 0x8000) as usize]
            }
            0x9800..=0x9FFF => {
                if addr - 0x9800 == 0x43 {
                    // panic!()
                }
                // println!("write to background map {:04x}", addr - 0x9800);
                &mut self.background_map[(addr - 0x9800) as usize]
            }
            0xA000..=0xBFFF => &mut self.cartridge_ram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => &mut self.wram[(addr - 0xC000) as usize],
            0xFE00..=0xFE9F => &mut self.sprite[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => &mut self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFF => &mut self.high_ram[(addr - 0xFF80) as usize],
            _ => {
                println!("Unused/unmapped memory");
                &mut self.unused_write_dummy
            }
        }
    }
}
