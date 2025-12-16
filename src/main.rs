#![no_main]
#![no_std]

extern crate alloc;

mod buffer;
mod rand;

use alloc::format;
use alloc::vec::Vec;
use core::time::Duration;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltPixel, GraphicsOutput};
use uefi::proto::console::text::{Key, ScanCode};
use uefi::{boot, Result};
use core::fmt::Write;

use crate::buffer::Buffer;
use crate::rand::Rng;

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: isize,
    pub y: isize,
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
    let mut length: usize = 1;

    let mut direction = "down";

    let mut fruits: Vec<Vec2> = Vec::new();

    // rng
    let mut rng = Rng::new();

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
                        fruits = Vec::new();
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
                Key::Special(ScanCode::ESCAPE) => {
                    length += 1;
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

            // check if player is eating fruit (compare in grid coordinates)
            let grid_pos_x = (player_x - offset_x) / block_size;
            let grid_pos_y = (player_y - offset_y) / block_size;
            if let Some(idx) = fruits
                .iter()
                .position(|f| f.x == grid_pos_x && f.y == grid_pos_y)
            {
                fruits.remove(idx);
                length += 1;
                system::with_stdout(|stdout| {
                    let _ = stdout.write_str("\x07");
                });
            }
        }

        // clear screen
        buffer.clear();

        // generate fruits (store in grid coordinates)
        if rng.random_bool(0.05) && fruits.len() < 2 && !dead {
            let blocks_grid: Vec<Vec2> = blocks
                .iter()
                .map(|b| Vec2 {
                    x: (b.x - offset_x) / block_size,
                    y: (b.y - offset_y) / block_size,
                })
                .collect();
            let mut occupied = blocks_grid;
            occupied.extend(fruits.iter().cloned());
            if let Some(pos) = rng.random_block(&occupied, grid_w as usize, grid_h as usize) {
                fruits.push(Vec2 {
                    x: pos.x as isize,
                    y: pos.y as isize,
                });
            }
        }

        // render fruits (convert grid to pixel coordinates when drawing)
        for fruit in &fruits {
            let pixel_x = fruit.x * block_size + offset_x;
            let pixel_y = fruit.y * block_size + offset_y;
            buffer.rectangle(
                (pixel_x + block_size / 4) as usize,
                (pixel_y + block_size / 4) as usize,
                block_size as usize / 2,
                block_size as usize / 2,
                BltPixel::new(220, 30, 70),
                true,
            );
        }

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

        let _ = Text::new(
            format!("Score: {}", (length - 1).max(0)).as_str(),
            Point::new(10, 20),
            MonoTextStyle::new(&FONT_10X20, Rgb888::new(30, 30, 255)),
        )
        .draw(&mut buffer);

        if dead {
            let style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 0, 0));

            let game_over_text = "Game Over!";
            let restart_text = "Press Space to restart";

            let game_over_width = game_over_text.len() as u32 * 10;
            let restart_width = restart_text.len() as u32 * 10;

            let game_over_x = (width / 2) as i32 - (game_over_width as i32 / 2);
            let restart_x = (width / 2) as i32 - (restart_width as i32 / 2);

            let _ = Text::new(
            game_over_text,
            Point::new(game_over_x, (height / 2 - 10) as i32),
            style,
            )
            .draw(&mut buffer);

            let _ = Text::new(
            restart_text,
            Point::new(restart_x, (height / 2 + 15) as i32),
            style,
            )
            .draw(&mut buffer);
        }

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
