use crate::joypad_input_handler::JoypadInputHandler;
use crate::memory::Memory;
use minifb::{Key, Scale, Window, WindowOptions};
use std::time::SystemTime;

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
    frame_start_time: SystemTime,
    in_vblank: bool,
    stat_line: bool,
}

impl PPU {
    pub fn new(ih: JoypadInputHandler) -> Self {
        let mut window = Window::new(
            "Pixel Grid - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X8,
                ..WindowOptions::default()
            },
        )
        .unwrap();
        window.set_input_callback(Box::new(ih));
        Self {
            sl: 0,
            dot: 0,
            line_idx: 0,
            buffer: [0; WIDTH * HEIGHT],
            window,
            oam_dma_start: 0xFF,
            oam_dma_ctr: OAM_DMA_LENGTH,
            sl_objects: [0; SPRITE_BYTES * 10],
            sl_objects_nxt: 0,
            frame_start_time: SystemTime::now(),
            in_vblank: false,
            stat_line: false,
        }
    }

    pub fn run_dot(&mut self, mut mem: Memory) -> Memory {
        let mut lcdc = LCDC::load(&mem);
        if !lcdc.lcd_ppu_enable {
            mem[0xFF41] &= 0b1111_1100;
            return mem;
        }

        self.oam_dma_transfer(&mut mem);

        mem[0xFF44] = self.sl;
        let ppu_mode = if self.sl < MODE_1_SL {
            self.in_vblank = false;
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
                    let mut pixel = 0u8;
                    if lcdc.bg_window_enable {
                        pixel = self.render_background_layer(&mem, self.line_idx, self.sl, &mut lcdc);
                        if lcdc.window_enable {
                            pixel =
                                self.render_window_layer(self.line_idx, self.sl, &mem, &mut lcdc)
                        }
                    }

                    if lcdc.obj_enable {
                        self.render_sprite_layer(&mut mem, &mut pixel);
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
            if !self.in_vblank {
                mem[0xFF0F] |= 0b0000_0001;
                self.in_vblank = true;
                self.render(&mem);
            }
            1
        };

        let mut new_stat_line = false;
        // Check LY == LYC
        new_stat_line |= mem[0xFF45] == mem[0xFF44];
        let lyc_eq: u8 = if mem[0xFF45] == mem[0xFF44] { 1 } else { 0 };
        let lcd_stat = lyc_eq << 2 | ppu_mode;
        mem[0xFF41] = mem[0xFF41] & 0b11111000 | lcd_stat;
        // Check mode triggers
        new_stat_line |= (mem[0xFF41] >> (ppu_mode + 3)) & 1 != 0;

        // Trigger stat interrupt on rising edge
        if !self.stat_line && new_stat_line {
            mem[0xFF0F] |= 1 << 1;
        }
        self.stat_line = new_stat_line;

        self.dot += 1;
        if self.dot >= TOTAL_DOTS {
            self.dot = 0;
            self.sl += 1;
            if self.sl >= TOTAL_SL {
                self.sl = 0;
                let now = SystemTime::now();
                println!(
                    "FPS {:?}",
                    1.0 / now
                        .duration_since(self.frame_start_time)
                        .unwrap()
                        .as_secs_f32()
                );
                self.frame_start_time = now;
            }
        }
        mem
    }

    fn render_background_layer(&self, mem: &Memory, x: u8, y: u8, lcdc: &mut LCDC) -> u8 {
        let scroll_x = mem[0xFF43];
        let scroll_y = mem[0xFF42];
        let x = x.wrapping_add(scroll_x);
        let y = y.wrapping_add(scroll_y);

        Self::get_pixel_from_tile_map(mem, lcdc, x, y, lcdc.bg_tile_map)
    }

    fn get_pixel_from_tile_map(mem: &Memory, lcdc: &LCDC, x: u8, y: u8, tile_map: bool) -> u8 {
        let tile_x = x / TILE_X;
        let tile_y = y / TILE_Y;
        let tile_coord_x = x % TILE_X;
        let tile_coord_y = y % TILE_Y;

        let tile_idx = if tile_map {
            // 9C00–9FFF
            mem[0x9C00 + tile_x as u16 + tile_y as u16 * TILE_TABLE_SIZE]
        } else {
            // 9800–9BFF
            mem[0x9800 + tile_x as u16 + tile_y as u16 * TILE_TABLE_SIZE]
        };
        let s = if lcdc.bg_window_tile_data_area {
            // 8000–8FFF unsigned
            0x8000 + tile_idx as u16 * TILE_SIZE_BYTES
        } else {
            // 8800–97FF signed
            0x9000u16.wrapping_add_signed((tile_idx as i8) as i16 * 16)
        };
        if tile_idx != 127 || true {
            // println!(
            //     "({}, {}) ({}, {}) ({}, {}) ({}, {}) {:04x} {} {}",
            //     win_x, win_y, x, y, tile_x, tile_y, tile_coord_x, tile_coord_y, 0x9800 + tile_x as u16 + tile_y as u16 * TILE_TABLE_SIZE, tile_idx, s
            // );
        }
        Self::get_pixel_in_tile(mem, tile_coord_x, tile_coord_y, s)
    }

    fn render_window_layer(&mut self, x: u8, y: u8, mem: &Memory, lcdc: &mut LCDC) -> u8 {
        let win_x = mem[0xFF4B];
        let win_y = mem[0xFF4A];
        let x = x.wrapping_add(win_x).wrapping_sub(7);
        let y = y.wrapping_add(win_y);
        Self::get_pixel_from_tile_map(mem, lcdc, x, y, lcdc.window_tile_map)
    }

    fn render_sprite_layer(&mut self, mem: &mut Memory, pixel: &mut u8) {
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
            let in_sprite_y = self.sl + 16 - sprite_y;
            let tile_start = 0x8000 + tile_idx as u16;

            sprite_min_x = sprite_x;
            *pixel = Self::get_pixel_in_tile(mem, in_sprite_x, in_sprite_y, tile_start);
        }
    }

    fn get_pixel_in_tile(mem: &Memory, in_tile_x: u8, in_tile_y: u8, tile_start: u16) -> u8 {
        let lo_channel = (mem[tile_start + (2 * in_tile_y as u16)] >> (7 - in_tile_x)) & 1;
        let hi_channel = (mem[tile_start + (2 * in_tile_y as u16 + 1)] >> (7 - in_tile_x)) & 1;
        let pixel_value = hi_channel << 1 | lo_channel;
        pixel_value
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
                // println!("COMPLETED OAM DMA")
            }
        }
    }

    fn render(&mut self, mem: &Memory) {
        if self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window
                .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
                .unwrap();
        }
    }

    fn set_pixel(&mut self, x: u8, y: u8, pixel: u8) {
        let grayscale = (pixel << 6) as u32;
        self.buffer[x as usize + y as usize * WIDTH] = grayscale << 16 | grayscale << 8 | grayscale;
    }
}
