use crate::cartridge_header::CartridgeHeader;
use log::error;
use std::cmp::{max, min};
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::ops::{Index, IndexMut};

pub const APU_READ_MASKS: [u8; 0x17] = [
    0b0111_1111, // FF10 - NR10
    0b1100_0000, // FF11 - NR11
    0b1111_1111, // FF12 - NR12
    0b0000_0000, // FF13 - NR13
    0b0100_0000, // FF14 - NR14
    0b0000_0000,
    0b1100_0000, // FF16 - NR21
    0b1111_1111, // FF17 - NR22
    0b0000_0000, // FF18 - NR23
    0b0100_0000, // FF19 - NR24
    0b1000_0000, // FF1A - NR30
    0x0000_0000, // FF1B - NR31
    0b0110_0000, // FF1C - NR32
    0x0000_0000, // FF1D - NR33
    0b0100_0000, // FF1E - NR34
    0b0000_0000,
    0b0000_0000, // FF20 - NR41
    0b1111_1111, // FF21 - NR42
    0b1111_1111, // FF22 - NR43
    0b0100_0000, // FF23 - NR44
    0b1111_1111, // FF24 - NR50
    0b1111_1111, // FF25 - NR51
    0b1000_1111, // FF26 - NR52
];

pub const APU_WRITE_MASKS: [u8; 0x17] = [
    0b0111_1111, // FF10 - NR10
    0b1111_1111, // FF11 - NR11
    0b1111_1111, // FF12 - NR12
    0b1111_1111, // FF13 - NR13
    0b1100_0111, // FF14 - NR14
    0b0000_0000,
    0b1111_1111, // FF16 - NR21
    0b1111_1111, // FF17 - NR22
    0b1111_1111, // FF18 - NR23
    0b1100_0111, // FF19 - NR24
    0b1000_0000, // FF1A - NR30
    0b1111_1111, // FF1B - NR31
    0b0110_0000, // FF1C - NR32
    0b1111_1111, // FF1D - NR33
    0b1100_0111, // FF1E - NR34
    0b0000_0000,
    0b1111_1111, // FF20 - NR41
    0b1111_1111, // FF21 - NR42
    0b1111_1111, // FF22 - NR43
    0b1100_0000, // FF23 - NR44
    0b1111_1111, // FF24 - NR50
    0b1111_1111, // FF25 - NR51
    0b1000_0000, // FF26 - NR52
];

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

    pub fn ram_bank(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug)]
enum MBC1MemoryModel {
    ROMUpperBits,
    RAMBanking,
    Neither,
}

pub struct MBC1 {
    memory_model: MBC1MemoryModel,
    ram_enable: u8,
    advanced_mode: u8, // last bit is: false == 16/8, true == 32/4
    rom_bank: u8,
    rom_bank_mask: u8,
    upper_rom_bank_bits: u8,
    ram_bank: u8,
    void: u8,
}

impl MBC1 {
    pub fn new(rom_size: u8, ram_size: u8) -> Self {
        // TODO this does not support the MBC1M Multi-carts.
        //  This will use 4 bits instead of 5 bits of the main rom bank and have the upper
        //  2 bits be bit 4 and 5 instead of 5 and 6.
        let memory_model = if rom_size >= 5 {
            MBC1MemoryModel::ROMUpperBits
        } else if ram_size >= 3 {
            MBC1MemoryModel::RAMBanking
        } else {
            MBC1MemoryModel::Neither
        };
        Self {
            memory_model,
            ram_enable: 0,
            advanced_mode: 0,
            rom_bank: 1,
            rom_bank_mask: min(0b0001_1111, 0b1111_1111 >> (8 - rom_size - 1)),
            upper_rom_bank_bits: 0,
            ram_bank: 0,
            void: 0,
        }
    }

    pub fn rom_bank(&self) -> usize {
        let lower_bank = max(self.rom_bank & self.rom_bank_mask, 1);
        let upper_bank = self.upper_rom_bank_bits & 0b11;
        let bank_number = lower_bank | (upper_bank << 5);
        max(bank_number, 1) as usize
    }

    pub fn rom_write(&mut self, addr: u16) -> &mut u8 {
        match addr {
            0x0000..=0x1FFF => &mut self.ram_enable,
            0x2000..=0x3FFF => &mut self.rom_bank,
            0x4000..=0x5FFF => {
                println!("Write to rom ram banking {:?}", self.memory_model);
                match self.memory_model {
                    MBC1MemoryModel::ROMUpperBits => &mut self.upper_rom_bank_bits,
                    MBC1MemoryModel::RAMBanking => &mut self.ram_bank,
                    MBC1MemoryModel::Neither => &mut self.void,
                }
            }
            0x6000..=0x7FFF => &mut self.advanced_mode,
            _ => panic!("Invalid write to memory bank: {:04x}", addr),
        }
    }

    pub fn ram_bank(&self) -> Option<usize> {
        if self.ram_enable == 0x0A {
            Some(self.ram_bank as usize)
        } else {
            None
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

    pub fn ram_bank(&self) -> Option<usize> {
        None
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

    pub fn ram_bank(&self) -> Option<usize> {
        match self {
            MemoryBankController::ROMOnly(c) => c.ram_bank(),
            MemoryBankController::MBC1(c) => c.ram_bank(),
            MemoryBankController::MBC3(c) => c.ram_bank(),
        }
    }
}

pub struct AudioRegisters {
    read: [u8; 0x17],
    internal: [u8; 0x17],
    write: [u8; 0x17],
}

impl Index<u16> for AudioRegisters {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.read[index as usize]
    }
}

impl IndexMut<u16> for AudioRegisters {
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        &mut self.write[index as usize]
    }
}

impl AudioRegisters {
    pub fn new() -> Self {
        Self {
            read: [0; 0x17],
            internal: [0; 0x17],
            write: [0; 0x17],
        }
    }

    pub fn update(&mut self) {
        let nr52_power_before = self.internal[0x16] & (1 << 7);
        if nr52_power_before == 0 {
            let old = self.write[0x16];
            self.write = [0; 0x17];
            self.write[0x16] = old;
        }

        for addr in 0usize..0x17 {
            self.update_bits(addr);
        }
    }

    fn update_bits(&mut self, addr: usize) {
        let write_bits = APU_WRITE_MASKS[addr] & self.write[addr];
        let no_write_bits = self.read[addr] & !APU_WRITE_MASKS[addr];
        self.internal[addr] = write_bits | no_write_bits;
        self.read[addr] = (APU_READ_MASKS[addr] & self.internal[addr]) | !APU_READ_MASKS[addr];
    }
}

pub struct Memory {
    bank_ctrl: MemoryBankController,
    num_rom_banks: usize,
    boot_rom: Vec<u8>,
    rom: Vec<u8>,
    pub tile_ram: [u8; 0x1800],
    pub background_map: [u8; 0x0800],
    cartridge_ram: [u8; 0x8000],
    wram: [u8; 0x2000],
    pub sprite: [u8; 0xA0],
    io1: [u8; 0x10],           // 00 - 0F
    pub audio: AudioRegisters, // 10 - 26
    wave_ram: [u8; 0x10],      // 30 - 3F
    io2: [u8; 0x37],           // 40 - 77
    pub high_ram: [u8; 0x89],  // 78 - FF
    unused_response: u8,
    unused_write_dummy: u8,
    audio_disabled: u8,
    void: u8,
}

impl Memory {
    pub fn new(boot_rom: Vec<u8>, rom: Vec<u8>, header: CartridgeHeader) -> Result<Self, String> {
        let mut mem = Self {
            bank_ctrl: header.memory_bank_controller()?,
            num_rom_banks: 2usize.pow(header.rom_size as u32 + 1),
            boot_rom,
            rom,
            tile_ram: [0; 0x1800],
            background_map: [0; 0x0800],
            cartridge_ram: [0; 0x8000],
            wram: [0; 0x2000],
            sprite: [0; 0xA0],
            io1: [0; 0x10],
            audio: AudioRegisters::new(),
            wave_ram: [0; 0x10],
            io2: [0; 0x37],
            high_ram: [0; 0x89],
            unused_response: 0xFF,
            unused_write_dummy: 0,
            audio_disabled: 0,
            void: 0xFF,
        };
        mem.audio[0xFF10 - 0xFF10] |= 0x80;
        Ok(mem)
    }

    pub fn write_contents(&self) -> io::Result<()> {
        BufWriter::new(File::create("./tile_ram.bin")?).write_all(&self.tile_ram)?;
        BufWriter::new(File::create("./background_map.bin")?).write_all(&self.background_map)?;
        BufWriter::new(File::create("./sprite.bin")?).write_all(&self.sprite)?;
        BufWriter::new(File::create("./high_ram.bin")?).write_all(&self.high_ram)?;
        // BufWriter::new(File::create("./audio.bin")?).write_all(&self.audio)?;
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
            0xA000..=0xBFFF => match self.bank_ctrl.ram_bank() {
                Some(bank) => &self.cartridge_ram[(addr - 0xA000) as usize + 0x2000 * bank],
                None => &self.void,
            },
            0xC000..=0xDFFF => &self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => &self.wram[(addr - 0xE000) as usize], // Echo RAM
            0xFE00..=0xFE9F => &self.sprite[(addr - 0xFE00) as usize],
            0xFF00..=0xFF0F => &self.io1[(addr - 0xFF00) as usize],
            0xFF10..=0xFF26 => &self.audio[addr - 0xFF10],
            0xFF30..=0xFF3F => &self.wave_ram[(addr - 0xFF30) as usize],
            0xFF40..=0xFF77 => &self.io2[(addr - 0xFF40) as usize],
            0xFF77..=0xFFFF => &self.high_ram[(addr - 0xFF77) as usize],
            _ => {
                println!("Unused memory read {:04x}", addr);
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
            0xA000..=0xBFFF => match self.bank_ctrl.ram_bank() {
                Some(bank) => &mut self.cartridge_ram[(addr - 0xA000) as usize + 0x2000 * bank],
                None => &mut self.void,
            },
            0xC000..=0xDFFF => &mut self.wram[(addr - 0xC000) as usize],
            0xFE00..=0xFE9F => &mut self.sprite[(addr - 0xFE00) as usize],
            0xFF00..=0xFF0F => &mut self.io1[(addr - 0xFF00) as usize],
            0xFF10..=0xFF26 => {
                // println!("write to audio {:04x}", addr);
                &mut self.audio[addr - 0xFF10]
            }
            0xFF30..=0xFF3F => &mut self.wave_ram[(addr - 0xFF30) as usize],
            0xFF40..=0xFF77 => &mut self.io2[(addr - 0xFF40) as usize],
            0xFF77..=0xFFFF => &mut self.high_ram[(addr - 0xFF77) as usize],
            _ => {
                println!("Unused memory write {:04x}", addr);
                &mut self.unused_write_dummy
            }
        }
    }
}
