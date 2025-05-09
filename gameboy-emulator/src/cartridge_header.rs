use crate::memory::{MemoryBankController, ROMOnly, MBC1, MBC3};

#[derive(Debug)]
pub struct CartridgeHeader {
    pub logo: [u8; 0x30],
    pub title: [u8; 0x10],
    pub manufacturer: [u8; 0x4],
    pub cgb: u8,
    pub licensee: [u8; 0x2],
    pub sgb: u8,
    pub cartridge_type: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub destination_code: u8,
    pub licensee_old: u8,
    pub rom_version: u8,
    pub header_checksum: u8,
    pub global_checksum: [u8; 2],
}

impl CartridgeHeader {
    pub fn read(rom: &Vec<u8>) -> Self {
        Self {
            logo: rom[0x104..=0x133].try_into().unwrap(),
            title: rom[0x134..=0x143].try_into().unwrap(),
            manufacturer: rom[0x13F..=0x142].try_into().unwrap(),
            cgb: rom[0x143],
            licensee: rom[0x144..=0x145].try_into().unwrap(),
            sgb: rom[0x146],
            cartridge_type: rom[0x147],
            rom_size: rom[0x148],
            ram_size: rom[0x149],
            destination_code: rom[0x14A],
            licensee_old: rom[0x14B],
            rom_version: rom[0x14C],
            header_checksum: rom[0x14D],
            global_checksum: rom[0x14E..=0x14F].try_into().unwrap(),
        }
    }

    pub fn memory_bank_controller(&self) -> Result<MemoryBankController, String> {
        match self.cartridge_type {
            0x00 => Ok(MemoryBankController::ROMOnly(ROMOnly::new())),
            0x01..=0x03 => Ok(MemoryBankController::MBC1(MBC1::new(self.rom_size, self.ram_size))),
            0x0F..=0x13 => Ok(MemoryBankController::MBC3(MBC3::new())),
            _ => Err("Cartridge type's MemoryBankController not implemented".to_string()),
        }
    }
}