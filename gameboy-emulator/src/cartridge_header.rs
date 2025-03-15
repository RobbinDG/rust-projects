#[derive(Debug)]
pub struct CartridgeHeader {
    logo: [u8; 0x30],
    title: [u8; 0x10],
    manufacturer: [u8; 0x4],
    cgb: u8,
    licensee: [u8; 0x2],
    sgb: u8,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    destination_code: u8,
    licensee_old: u8,
    rom_version: u8,
    header_checksum: u8,
    global_checksum: [u8; 2],
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
}