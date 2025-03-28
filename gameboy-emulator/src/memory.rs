use crate::cartridge_header::CartridgeHeader;
use std::cmp::max;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::ops::{Index, IndexMut};
use log::error;

pub struct ROMOnly {
    null: u8,
}

impl ROMOnly {
    pub fn new() -> Self {
        Self { null: 0 }
    }

    pub fn rom_bank(&self) -> usize {
        1
    }

    pub fn rom_write(&mut self, _: u16) -> &mut u8 {
        &mut self.null
    }
}

pub struct MBC1 {
    ram_bank_enable: u8,
    memory_model: u8, // last bit is: false == 16/8, true == 32/4
    rom_bank: u8,
    upper_rom_bank_bits: u8,
    ram_bank: u8,
}

impl MBC1 {
    pub fn new() -> Self {
        Self {
            ram_bank_enable: 0,
            memory_model: 0,
            rom_bank: 1,
            upper_rom_bank_bits: 0,
            ram_bank: 0,
        }
    }

    pub fn rom_bank(&self) -> usize {
        let bank_number = (self.rom_bank & 0b0001_1111) | ((self.upper_rom_bank_bits & 0b11) << 5);
        max(bank_number, 1) as usize
    }

    pub fn rom_write(&mut self, addr: u16) -> &mut u8 {
        match addr {
            0x0000..=0x1FFF => {
                if self.memory_model & 1 == 1 {
                    &mut self.ram_bank_enable
                } else {
                    println!("Memory model not set to 32/4");
                    &mut self.ram_bank_enable
                }
            }
            0x2000..=0x3FFF => &mut self.rom_bank,
            0x4000..=0x5FFF => {
                if self.memory_model & 1 == 1 {
                    // 32/4
                    &mut self.ram_bank
                } else {
                    // 16/8
                    &mut self.upper_rom_bank_bits
                }
            }
            0x6000..=0x7FFF => &mut self.memory_model,
            _ => panic!("Invalid write to memory bank: {:04x}", addr),
        }
    }
}

pub struct MBC3 {
    rom_bank_reg: u8,
}

impl MBC3 {
    pub fn new() -> Self {
        Self { rom_bank_reg: 1 }
    }

    pub fn rom_bank(&self) -> usize {
        max(self.rom_bank_reg, 1) as usize
    }

    pub fn rom_write(&mut self, addr: u16) -> &mut u8 {
        if !matches!(addr, 0x2000..=0x3FFF) {
            error!("Invalid write to memory bank: {:04x}", addr);
        }
        &mut self.rom_bank_reg
    }
}

pub enum MemoryBankController {
    ROMOnly(ROMOnly),
    MBC1(MBC1),
    MBC3(MBC3),
}

impl MemoryBankController {
    pub fn rom_bank(&self) -> usize {
        match self {
            MemoryBankController::ROMOnly(c) => c.rom_bank(),
            MemoryBankController::MBC1(c) => c.rom_bank(),
            MemoryBankController::MBC3(c) => c.rom_bank(),
        }
    }

    pub fn rom_write(&mut self, addr: u16) -> &mut u8 {
        match self {
            MemoryBankController::ROMOnly(c) => c.rom_write(addr),
            MemoryBankController::MBC1(c) => c.rom_write(addr),
            MemoryBankController::MBC3(c) => c.rom_write(addr),
        }
    }
}

pub struct Memory {
    bank_ctrl: MemoryBankController,
    num_rom_banks: usize,
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
    pub fn new(boot_rom: Vec<u8>, rom: Vec<u8>, header: CartridgeHeader) -> Result<Self, String> {
        Ok(Self {
            bank_ctrl: header.memory_bank_controller()?,
            num_rom_banks: 2usize.pow(header.rom_size as u32 + 1),
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
        })
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
            0x4000..=0x7FFF => {
                let bank = self.bank_ctrl.rom_bank();
                if bank >= self.num_rom_banks {
                    panic!("Selected bank {bank} not available.")
                }
                &self.rom[bank * 0x4000 + (addr as usize - 0x4000)]
            }
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
