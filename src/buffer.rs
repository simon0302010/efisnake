use alloc::vec;
use alloc::vec::Vec;
use uefi::{
    proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput},
    Result,
};

use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

pub struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            pixels: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.pixels {
            *pixel = BltPixel::new(0, 0, 0);
        }
    }

    fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels.get_mut(y * self.width + x)
    }

    pub fn rectangle(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: BltPixel,
        fill: bool,
    ) {
        for dy in 0..h {
            for dx in 0..w {
                let should_draw = fill || (dy == 0 || dy == h - 1) || (dx == 0 || dx == w - 1);
                if should_draw {
                    if let Some(pixel) = self.pixel(x + dx, y + dy) {
                        *pixel = color;
                    }
                }
            }
        }
    }

    pub fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }
}

impl OriginDimensions for Buffer {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl DrawTarget for Buffer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> core::result::Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.y >= 0 {
                if let Some(p) = self.pixel(point.x as usize, point.y as usize) {
                    *p = BltPixel::new(color.r(), color.g(), color.b());
                }
            }
        }
        Ok(())
    }
}
