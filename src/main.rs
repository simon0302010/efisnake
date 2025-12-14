#![no_main]
#![no_std]

mod rand;

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::console::text::{Key, ScanCode};
use uefi::{boot, Result};

use crate::rand::Rng;

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

    #[allow(unused)]
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
                } else if (dy == 0 || dy == h - 1) && (dx == 0 || dx == w - 1) {
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

    // stuff for square
    let rect_w: isize = 100;
    let rect_h: isize = 100;
    let mut rect_x = (width_i - rect_w) / 2;
    let mut rect_y = (height_i - rect_h) / 2;

    let speed: isize = 30;

    let mut rng = Rng::new();

    loop {
        if let Ok(Some(key)) = system::with_stdin(|stdin| stdin.read_key()) {
            match key {
                Key::Printable(c) => {
                    if c == uefi::Char16::try_from('q').unwrap_or_default()
                        || c == uefi::Char16::try_from('Q').unwrap_or_default()
                    {
                        break;
                    }
                }
                Key::Special(ScanCode::UP) => {
                    rect_y = (rect_y - speed).clamp(0, height_i - rect_h);
                }
                Key::Special(ScanCode::DOWN) => {
                    rect_y = (rect_y + speed).clamp(0, height_i - rect_h);
                }
                Key::Special(ScanCode::RIGHT) => {
                    rect_x = (rect_x + speed).clamp(0, width_i - rect_w);
                }
                Key::Special(ScanCode::LEFT) => {
                    rect_x = (rect_x - speed).clamp(0, width_i - rect_w);
                }
                _ => {}
            }
        }

        let r = rng.random_range(0, 255) as u8;
        let g = rng.random_range(0, 255) as u8;
        let b = rng.random_range(0, 255) as u8;

        buffer.clear();
        buffer.rectangle(
            rect_x as usize,
            rect_y as usize,
            rect_w as usize,
            rect_h as usize,
            BltPixel::new(r, g, b),
            true,
        );
        buffer.blit(&mut gop)?;

        boot::stall(Duration::from_millis(30));
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    game().unwrap();
    Status::SUCCESS
}
