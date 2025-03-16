use std::cmp::{max, min};
use std::ops::{Index, IndexMut};

pub struct Memory {
    rom: Vec<u8>,
    pub rom_bank_reg: u8,
    pub tile_ram: [u8; 0x1800],
    pub background_map: [u8; 0x0800],
    cartridge_ram: [u8; 0x2000],
    wram: [u8; 0x2000],
    pub sprite: [u8; 0xA0],
    io: [u8; 0x80],
    high_ram: [u8; 0x7F],
    ime: u8,
}

impl Memory {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            rom_bank_reg: 1,
            tile_ram: [0; 0x1800],
            background_map: [0; 0x0800],
            cartridge_ram: [0; 0x2000],
            wram: [0; 0x2000],
            sprite: [0; 0xA0],
            io: [0; 0x80],
            high_ram: [0; 0x7F],
            ime: 0,
        }
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, addr: u16) -> &Self::Output {
        match addr {
            0x0000..=0x3FFF => &self.rom[addr as usize],
            0x4000..=0x7FFF => &self.rom[(max(self.rom_bank_reg, 1) as u32 * 0x4000 + (addr as u32 - 0x4000)) as usize],
            0x8000..=0x97FF => &self.tile_ram[(addr - 0x8000) as usize],
            0x9800..=0x9FFF => &self.background_map[(addr - 0x9800) as usize],
            0xA000..=0xBFFF => &self.cartridge_ram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => &self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => &self.wram[(addr - 0xE000) as usize], // Echo RAM
            0xFE00..=0xFE9F => &self.sprite[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => &self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFE => &self.high_ram[(addr - 0xFF80) as usize],
            0xFFFF => &self.ime,
            _ => panic!("Unused memory"),
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        match addr {
            0x2000..=0x3FFF => &mut self.rom_bank_reg,
            0x8000..=0x97FF => {
                // println!("write to tile ram {:04x}", addr);
                &mut self.tile_ram[(addr - 0x8000) as usize]
            },
            0x9800..=0x9FFF => &mut self.background_map[(addr - 0x9800) as usize],
            0xA000..=0xBFFF => &mut self.cartridge_ram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => &mut self.wram[(addr - 0xC000) as usize],
            0xFE00..=0xFE9F => &mut self.sprite[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => {
                println!("Writing flag {:02x}", addr - 0xFF00);
                &mut self.io[(addr - 0xFF00) as usize]
            },
            0xFF80..=0xFFFE => &mut self.high_ram[(addr - 0xFF80) as usize],
            0xFFFF => &mut self.ime,
            _ => panic!("Unused/unmapped memory"),
        }
    }
}