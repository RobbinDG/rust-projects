use std::ops::{Index, IndexMut};

const APU_READ_MASKS: [u8; 0x17] = [
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

const APU_WRITE_MASKS: [u8; 0x17] = [
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

pub struct InternalAudioRegisters {
    internal: [u8; 0x17]
}

impl InternalAudioRegisters {
    pub fn new() -> Self {
        Self {
            internal: [0; 0x17]
        }
    }
}

impl Index<u16> for InternalAudioRegisters {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.internal[index as usize - 0xFF10]
    }
}

impl IndexMut<u16> for InternalAudioRegisters {
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        &mut self.internal[index as usize - 0xFF10]
    }
}

pub struct AudioRegisters {
    read: [u8; 0x17],
    pub internal: InternalAudioRegisters,
    write: [u8; 0x17],
}

impl Index<u16> for AudioRegisters {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.read[index as usize - 0xFF10]
    }
}

impl IndexMut<u16> for AudioRegisters {
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        &mut self.write[index as usize - 0xFF10]
    }
}

impl AudioRegisters {
    pub fn new() -> Self {
        Self {
            read: [0; 0x17],
            internal: InternalAudioRegisters::new(),
            write: [0; 0x17],
        }
    }

    pub fn update(&mut self) {
        let nr52_power_before = self.internal[0xFF26] & (1 << 7);
        if nr52_power_before == 0 {
            let old = self.write[0x16];
            self.write = [0; 0x17];
            self.write[0x16] = old;
        }

        for addr in 0xFF10..=0xFF26 {
            self.update_bits(addr);
        }
    }

    fn update_bits(&mut self, addr_orig: u16) {
        let addr = addr_orig as usize - 0xFF10;

        self.internal[addr_orig] = match addr_orig {
            0xFF10..=0xFF13 | 0xFF15..=0xFF26 => {
                let write_bits = APU_WRITE_MASKS[addr] & self.write[addr];
                let no_write_bits = self.internal[addr_orig] & !APU_WRITE_MASKS[addr];
                write_bits | no_write_bits
            },
            0xFF14 => {
                let write_bits = APU_WRITE_MASKS[addr] & self.write[addr];
                let no_write_bits = self.internal[addr_orig] & !APU_WRITE_MASKS[addr];
                self.write[addr] &= !(1 << 7);
                write_bits | no_write_bits
            }
            _ => panic!("{:04x} is not an audio register", addr_orig),
        };
        self.read[addr] = (APU_READ_MASKS[addr] & self.internal[addr_orig]) | !APU_READ_MASKS[addr];
    }
}
