#![no_main]
#![no_std]

mod misc;
mod buffer;

extern crate alloc;

use alloc::vec::Vec;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use core::time::Duration;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltPixel, GraphicsOutput};
use uefi::proto::console::text::{Key, ScanCode};
use uefi::{boot, Result};

use crate::buffer::Buffer;
use crate::misc::Vec2;

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
    let mut length: usize = 1;

    let mut direction = "down";

    let score_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 100, 255));

    // will be used later
    // let mut rng = Rng::new();

    let mut running = true;
    let mut dead = false;

    while running {
        while let Ok(Some(key)) = system::with_stdin(|stdin| stdin.read_key()) {
            match key {
                Key::Printable(c) => {
                    if c == uefi::Char16::try_from('q').unwrap_or_default()
                        || c == uefi::Char16::try_from('Q').unwrap_or_default()
                    {
                        running = false;
                    } else if c == uefi::Char16::try_from(' ').unwrap_or_default() && dead {
                        dead = false;
                        blocks = Vec::new();
                        length = 1;
                        direction = "down";
                        player_x = ((grid_w / 2) * block_size) + offset_x;
                        player_y = ((grid_h / 2) * block_size) + offset_y;
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
        }

        // movement logic
        if !dead {
            let grid_pos_x = (player_x - offset_x) / block_size;
            let grid_pos_y = (player_y - offset_y) / block_size;

            match direction {
                "up" => {
                    let new_pos = (grid_pos_y - 1).clamp(0, grid_h - 1);
                    if new_pos != (grid_pos_y - 1) {
                        dead = true;
                    }
                    player_y = new_pos * block_size + offset_y;
                }
                "down" => {
                    let new_pos = (grid_pos_y + 1).clamp(0, grid_h - 1);
                    if new_pos != (grid_pos_y + 1) {
                        dead = true;
                    }
                    player_y = new_pos * block_size + offset_y;
                }
                "right" => {
                    let new_pos = (grid_pos_x + 1).clamp(0, grid_w - 1);
                    if new_pos != (grid_pos_x + 1) {
                        dead = true;
                    }
                    player_x = new_pos * block_size + offset_x;
                }
                "left" => {
                    let new_pos = (grid_pos_x - 1).clamp(0, grid_w - 1);
                    if new_pos != (grid_pos_x - 1) {
                        dead = true;
                    }
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
                .any(|block| block.x == player_x && block.y == player_y)
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
            // color based on dead or not and head or not
            let color = if !dead {
                if i == blocks.len() - 1 {
                    BltPixel::new(0, 200, 0)
                } else {
                    BltPixel::new(0, 100, 0)
                }
            } else if i == blocks.len() - 1 {
                BltPixel::new(200, 0, 0)
            } else {
                BltPixel::new(100, 0, 0)
            };

            buffer.rectangle(
                block.x as usize,
                block.y as usize,
                block_size as usize,
                block_size as usize,
                color,
                true,
            );
        }

        let _ = Text::new("Score", Point::new(10, 20), score_style).draw(&mut buffer);

        buffer.blit(&mut gop)?;

        boot::stall(Duration::from_millis(125));
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
