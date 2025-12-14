#![no_main]
#![no_std]

mod misc;

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::console::text::{Key, ScanCode};
use uefi::{boot, Result};

use crate::misc::Vec2;

struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            pixels: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    fn clear(&mut self) {
        for pixel in &mut self.pixels {
            *pixel = BltPixel::new(0, 0, 0);
        }
    }

    fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels.get_mut(y * self.width + x)
    }

    fn rectangle(&mut self, x: usize, y: usize, w: usize, h: usize, color: BltPixel, fill: bool) {
        for dy in 0..h {
            for dx in 0..w {
                if fill {
                    if let Some(pixel) = self.pixel(x + dx, y + dy) {
                        *pixel = color;
                    }
                } else if (dy == 0 || dy == h - 1) || (dx == 0 || dx == w - 1) {
                    if let Some(pixel) = self.pixel(x + dx, y + dy) {
                        *pixel = color;
                    }
                }
            }
        }
    }

    fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }
}

fn game() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let (width_i, height_i) = (width as isize, height as isize);

    let block_size = (width_i.min(height_i)) / 20;
    let grid_w = width_i / block_size;
    let grid_h = height_i / block_size;
    let offset_x = (width_i - grid_w * block_size) / 2;
    let offset_y = (height_i - grid_h * block_size) / 2;

    let mut player_x = ((grid_w / 2) * block_size) + offset_x;
    let mut player_y = ((grid_h / 2) * block_size) + offset_y;

    // list of blocks
    let mut blocks: Vec<Vec2> = Vec::new();
    let mut length: usize = 10;

    let mut direction = "down";

    // will be used later
    // let mut rng = Rng::new();

    let mut running = true;
    let mut dead = false;

    while running {
        loop {
            if let Ok(Some(key)) = system::with_stdin(|stdin| stdin.read_key()) {
                match key {
                    Key::Printable(c) => {
                        if c == uefi::Char16::try_from('q').unwrap_or_default()
                            || c == uefi::Char16::try_from('Q').unwrap_or_default()
                        {
                            running = false;
                        }
                    }
                    Key::Special(ScanCode::UP) => {
                        if direction != "down" {
                            direction = "up";
                        }
                    }
                    Key::Special(ScanCode::DOWN) => {
                        if direction != "up" {
                            direction = "down";
                        }
                    }
                    Key::Special(ScanCode::RIGHT) => {
                        if direction != "left" {
                            direction = "right";
                        }
                    }
                    Key::Special(ScanCode::LEFT) => {
                        if direction != "right" {
                            direction = "left";
                        }
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        // movement logic
        if !dead {
            let grid_pos_x = (player_x - offset_x) / block_size;
            let grid_pos_y = (player_y - offset_y) / block_size;

            match direction {
                "up" => {
                    let new_pos = (grid_pos_y - 1).clamp(0, grid_h - 1);
                    player_y = new_pos * block_size + offset_y;
                }
                "down" => {
                    let new_pos = (grid_pos_y + 1).clamp(0, grid_h - 1);
                    player_y = new_pos * block_size + offset_y;
                }
                "right" => {
                    let new_pos = (grid_pos_x + 1).clamp(0, grid_w - 1);
                    player_x = new_pos * block_size + offset_x;
                }
                "left" => {
                    let new_pos = (grid_pos_x - 1).clamp(0, grid_w - 1);
                    player_x = new_pos * block_size + offset_x;
                }
                _ => {}
            }

            // only remove block if snake is too long
            if blocks.len() >= length {
                blocks.remove(0);
            }

            // there is probably a better way to do this
            if blocks
                .iter()
                .find(|block| block.x == player_x && block.y == player_y)
                .is_some()
            {
                dead = true;
            }

            // add head
            blocks.push(Vec2 {
                x: player_x,
                y: player_y,
            });
        }

        // clear screen
        buffer.clear();

        // draw all blocks
        for (i, block) in blocks.iter().enumerate() {
            let color: BltPixel;
            if !dead {
                color = if i == blocks.len() - 1 {
                    BltPixel::new(0, 200, 0)
                } else {
                    BltPixel::new(0, 100, 0)
                };
            } else {
                color = if i == blocks.len() - 1 {
                    BltPixel::new(200, 0, 0)
                } else {
                    BltPixel::new(100, 0, 0)
                };
            }

            buffer.rectangle(
                block.x as usize,
                block.y as usize,
                block_size as usize,
                block_size as usize,
                color,
                true,
            );
        }
        
        buffer.blit(&mut gop)?;

        boot::stall(Duration::from_millis(200));

        // temporary
        if dead {
            boot::stall(Duration::from_secs(3));
            running = false;
        }
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
