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
const OAM_DMA_LENGTH: u16 = 160;
const SPRITE_BYTES: usize = 4;
const SL_SPRITE_CAPACITY: usize = 10;

struct LCDC {
    lcd_ppu_enable: bool,
    window_tile_map: bool,
    window_enable: bool,
    bg_window_tile_data_area: bool,
    bg_tile_map: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_window_enable: bool,
}

impl LCDC {
    pub fn load(mem: &Memory) -> Self {
        let lcdc = mem[0xFF40];
        Self {
            lcd_ppu_enable: lcdc & (1 << 7) != 0,
            window_tile_map: lcdc & (1 << 6) != 0,
            window_enable: lcdc & (1 << 5) != 0,
            bg_window_tile_data_area: lcdc & (1 << 4) != 0,
            bg_tile_map: lcdc & (1 << 3) != 0,
            obj_size: lcdc & (1 << 2) != 0,
            obj_enable: lcdc & (1 << 1) != 0,
            bg_window_enable: lcdc & (1 << 0) != 0,
        }
    }
}

pub struct PPU {
    sl: u8,
    dot: u16,
    line_idx: u8,
    buffer: [u32; WIDTH * HEIGHT],
    window: Window,
    oam_dma_start: u16,
    oam_dma_ctr: u16,
    sl_objects: [u8; SPRITE_BYTES * 10],
    sl_objects_nxt: usize,
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
            oam_dma_start: 0xFF,
            oam_dma_ctr: OAM_DMA_LENGTH,
            sl_objects: [0; SPRITE_BYTES * 10],
            sl_objects_nxt: 0,
        }
    }

    pub fn run_dot(&mut self, mut mem: Memory) -> Memory {
        let mut lcdc = LCDC::load(&mem);
        if !lcdc.lcd_ppu_enable {
            return mem;
        }

        self.oam_dma_transfer(&mut mem);

        mem[0xFF44] = self.sl;
        let ppu_mode = if self.sl < MODE_1_SL {
            if self.dot < MODE_0_DOTS {
                // Mode 0: OAM scan
                self.line_idx = 0;
                // 40 sprites to check, 80 dots.
                if self.dot % 2 == 0 && self.sl_objects_nxt < SL_SPRITE_CAPACITY {
                    let sprite_idx = 0xFE00 + self.dot * (SPRITE_BYTES as u16) / 2;
                    let sprite_y = mem[sprite_idx + 0];
                    // TODO thet sl + 16 here depends on the mode, the one with sprite_y doesn't
                    if sprite_y <= self.sl + 16 && self.sl + 16 < sprite_y + 16 {
                        // Copy sprite data if it is in the scanline.
                        self.sl_objects[self.sl_objects_nxt + 0] = mem[sprite_idx + 0];
                        self.sl_objects[self.sl_objects_nxt + 1] = mem[sprite_idx + 1];
                        self.sl_objects[self.sl_objects_nxt + 2] = mem[sprite_idx + 2];
                        self.sl_objects[self.sl_objects_nxt + 3] = mem[sprite_idx + 3];
                        self.sl_objects_nxt += SPRITE_BYTES;
                    }
                }
                0
            } else if self.dot < MODE_0_DOTS + MODE_2_MIN_DOTS {
                // Mode 2: Drawing
                self.line_idx += 1;
                if (self.line_idx as usize) < WIDTH {
                    // Load BG
                    let mut pixel = self.get_bg_pixel_for(&mem, self.line_idx, self.sl, &mut lcdc);

                    // Load sprites
                    let mut sprite_min_x = WIDTH as u8 + 1;
                    for i in (0..self.sl_objects_nxt).step_by(SPRITE_BYTES) {
                        let sprite_y = self.sl_objects[i + 0];
                        let sprite_x = self.sl_objects[i + 1];
                        let tile_idx = self.sl_objects[i + 2];
                        let sprite_attrs = self.sl_objects[i + 3];

                        if !(sprite_x <= self.line_idx + 8 && self.line_idx + 8 < sprite_x + 8) {
                            continue;
                        }

                        if sprite_x >= sprite_min_x {
                            continue;
                        }

                        let in_sprite_x = self.line_idx + 8 - sprite_x;
                        // println!("Drawing sprite: x {} {} {} y {} {}", self.line_idx, sprite_x, in_sprite_x, self.sl, sprite_y);
                        let in_sprite_y = (self.sl + 16 - sprite_y) as u16;
                        let tile_start = 0x8000 + tile_idx as u16;

                        let lo_channel = (mem[tile_start + (2 * in_sprite_y)] >> in_sprite_x) & 1;
                        let hi_channel = (mem[tile_start + (2 * in_sprite_y + 1)] >> in_sprite_x) & 1;
                        pixel = hi_channel << 1 | lo_channel;
                        sprite_min_x = sprite_x;
                    }
                    // println!("({}, {}) = {}  (sprites: {})", self.line_idx, self.sl, pixel, self.sl_objects_nxt);
                    self.set_pixel(self.line_idx, self.sl, pixel);
                }
                2
            } else {
                // Mode 3: Horizontal blank
                self.sl_objects_nxt = 0;
                3
            }
        } else {
            // Mode 1: Vertical blank
            1
        };

        let lyc_eq: u8 = if mem[0xFF45] == mem[0xFF44] { 1 } else { 0 };
        let lcd_stat = lyc_eq << 2 | ppu_mode;
        mem[0xFF41] = mem[0xFF41] & 0b11111000 | lcd_stat;

        self.dot += 1;
        if self.dot >= TOTAL_DOTS {
            self.dot = 0;
            self.sl += 1;
            self.render(&mem);
            if self.sl >= TOTAL_SL {
                self.sl = 0;
            }
        }
        mem
    }

    fn oam_dma_transfer(&mut self, mem: &mut Memory) {
        let reg = mem[0xFF46];
        if reg <= 0xDF {
            self.oam_dma_start = (reg as u16) << 8;
            self.oam_dma_ctr = 0;
            mem[0xFF46] = 0xFF;
        }
        if self.oam_dma_ctr < OAM_DMA_LENGTH {
            let src = self.oam_dma_start | self.oam_dma_ctr;
            let dst = 0xFE00 | self.oam_dma_ctr;
            mem[dst] = mem[src];
            self.oam_dma_ctr += 1;
            if self.oam_dma_ctr == OAM_DMA_LENGTH {
                println!("COMPLETED OAM DMA")
            }
        }
    }

    fn render(&mut self, mem: &Memory) {
        // Rendering

        if self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            // for x in 0..WIDTH {
            //     for y in 0..HEIGHT {
            //         self.set_bg_pixel_for(mem, x as u8, y as u8);
            //     }
            // }

            self.window
                .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
                .unwrap();
        }
    }

    fn get_bg_pixel_for(&self, mem: &Memory, x: u8, y: u8, lcdc: &mut LCDC) -> u8 {
        let scroll_x = mem[0xFF43];
        let scroll_y = mem[0xFF42];
        let x = x + scroll_x;
        let y = y + scroll_y;
        let tile_x = x / TILE_X;
        let tile_y = y / TILE_Y;
        let tile_coord_x = x % TILE_X;
        let tile_coord_y = y % TILE_Y;

        let tile_idx = if lcdc.bg_tile_map {
            // 9C00–9FFF
            mem[0x9C00 + tile_x as u16 * TILE_TABLE_SIZE + tile_y as u16]
        } else {
            // 9800–9BFF
            mem[0x9800 + tile_x as u16 * TILE_TABLE_SIZE + tile_y as u16]
        };
        let s = if lcdc.bg_window_tile_data_area {
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
        hi_channel << 1 | lo_channel
    }

    fn set_pixel(&mut self, x: u8, y: u8, pixel: u8) {
        let grayscale = (pixel << 6) as u32;
        self.buffer[x as usize * HEIGHT + y as usize] =
            grayscale << 16 | grayscale << 8 | grayscale;
    }
}
