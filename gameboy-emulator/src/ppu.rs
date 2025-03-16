use crate::memory::Memory;
use minifb::{Key, Window, WindowOptions};


const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const TILE_X: u8 = 8;
const TILE_Y: u8 = 8;
const TILE_TABLE_SIZE: u16 = 32;
const TILE_SIZE_BYTES: u16 = 16;

const MODE_0_DOTS: u16 = 80;
const MODE_1_SL: u8 = 144;
const MODE_2_MIN_DOTS: u16 = 172;
const TOTAL_DOTS: u16 = 456;
const TOTAL_SL: u8 = 154;

pub struct PPU {
    sl: u8,
    dot: u16,
    line_idx: u8,
    buffer: [u32; WIDTH * HEIGHT],
    window: Window,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            sl: 0,
            dot: 0,
            line_idx: 0,
            buffer: [0; WIDTH * HEIGHT],
            window: Window::new(
                "Pixel Grid - ESC to exit",
                WIDTH,
                HEIGHT,
                WindowOptions::default(),
            )
            .unwrap(),
        }
    }

    pub fn run_dot(&mut self, mut mem: Memory) -> Memory {
        mem[0xFF44] = self.sl;
        let ppu_mode = if self.sl < MODE_1_SL {
            if self.dot < MODE_0_DOTS {
                // Mode 0: OAM scan
                self.line_idx = 0;
                0
            } else if self.dot < MODE_0_DOTS + MODE_2_MIN_DOTS {
                // Mode 2: Drawing
                self.line_idx += 1;
                if (self.line_idx as usize) < WIDTH {
                    self.set_bg_pixel_for(&mem, self.line_idx, self.sl);
                }
                2
            } else {
                // Mode 3: Horizontal blank
                3
            }
        } else {
            // Mode 1: Vertical blank
            1
        };
        let lyc_eq: u8 = if mem[0xFF45] == mem[0xFF44] { 1 } else { 0 };
        let lcd_stat = lyc_eq << 2 | ppu_mode;
        mem[0xFF46] = mem[0xFF46] & 0b11111000 | lcd_stat;

        self.dot += 1;
        if self.dot >= TOTAL_DOTS {
            self.dot = 0;
            self.sl += 1;
            // println!("sl {}", self.sl);
            self.render(&mem);
            if self.sl >= TOTAL_SL {
                self.sl = 0;
            }
        }
        mem
    }

    fn render(&mut self, mem: &Memory) {
        // Rendering



        if self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            // for x in 0..WIDTH {
            //     for y in 0..HEIGHT {
            //         self.set_bg_pixel_for(mem, x as u8, y as u8);
            //     }
            // }

            self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT).unwrap();
        }
    }

    fn set_bg_pixel_for(&mut self, mem: &Memory, x: u8, y: u8) {
        let lcdc = mem[0xFF40];
        let lcd_ppu_enable = lcdc & (1 << 7) != 0;
        let window_tile_map = lcdc & (1 << 6) != 0;
        let window_enable = lcdc & (1 << 5) != 0;
        let bg_window_tile_data_area = lcdc & (1 << 4) != 0;
        let bg_tile_map = lcdc & (1 << 3) != 0;
        let obj_size = lcdc & (1 << 2) != 0;
        let obj_enable = lcdc & (1 << 1) != 0;
        let bg_window_enable = lcdc & (1 << 0) != 0;

        let scroll_x = mem[0xFF43];
        let scroll_y = mem[0xFF42];
        let x = x + scroll_x;
        let y = y + scroll_y;
        let tile_x = x / TILE_X;
        let tile_y = y / TILE_Y;
        let tile_coord_x = x % TILE_X;
        let tile_coord_y = y % TILE_Y;

        let tile_idx = if bg_tile_map {
            // 9C00–9FFF
            mem[0x9C00 + tile_x as u16 * TILE_TABLE_SIZE + tile_y as u16]
        } else {
            // 9800–9BFF
            mem[0x9800 + tile_x as u16 * TILE_TABLE_SIZE + tile_y as u16]
        };
        let s = if bg_window_tile_data_area {
            // 8000–8FFF unsigned
            0x8000 + tile_idx as u16 * TILE_SIZE_BYTES
        } else {
            // 8800–97FF signed
            0x8800u16.wrapping_add_signed((tile_idx as i8) as i16 * 32)
        };
        let mut tile = [0; 32];
        for i in 0u16..32 {
            tile[i as usize] = mem[s + i];
        }

        let hi_channel = (tile[(2 * tile_coord_y) as usize] >> tile_coord_x) & 1;
        let lo_channel = (tile[(2 * tile_coord_y + 1) as usize] >> tile_coord_x) & 1;
        let colour = hi_channel << 1 | lo_channel;

        let grayscale = (colour << 6) as u32;
        self.buffer[x as usize * HEIGHT + y as usize] = grayscale << 16 | grayscale << 8 | grayscale;
    }
}
