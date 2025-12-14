use alloc::vec::Vec;
use uefi::boot;
use uefi::proto::rng::Rng as EfiRng;

use crate::Vec2;

pub struct Rng {
    pub state: f64,
}

impl Rng {
    /// creates a new rng
    pub fn new() -> Self {
        let seed = match boot::get_handle_for_protocol::<EfiRng>() {
            Ok(handle) => match boot::open_protocol_exclusive::<EfiRng>(handle) {
                Ok(mut rng) => {
                    let mut buf = [0u8; 8];
                    if rng.get_rng(None, &mut buf).is_ok() {
                        u64::from_le_bytes(buf) as f64
                    } else {
                        12345678901234567.0
                    }
                }
                Err(_) => 12345678901234567.0,
            },
            Err(_) => 12345678901234567.0,
        };

        Self { state: seed }
    }

    // generates a random integer in the specified range
    pub fn random_range(&mut self, min: i64, max: i64) -> i64 {
        let range = max - min;
        min + (self.random_float() * range as f64) as i64
    }

    /// generates a random float from 0.0 to 1.0
    pub fn random_float(&mut self) -> f64 {
        let a: i64 = 6364136223846793005;
        let c: i64 = 1442695040888963407;

        let next = a.wrapping_mul(self.state as i64).wrapping_add(c);
        self.state = next.abs() as f64;
        self.state / core::i64::MAX as f64
    }

    pub fn random_bool(&mut self, p: f64) -> bool {
        self.random_float() <= p
    }

    pub fn random_block(&mut self, blocks: &Vec<Vec2>, w: usize, h: usize) -> Option<Vec2> {
        if blocks.len() >= (w * h) {
            return None;
        }

        for _ in 0..100 {
            let x = self.random_range(0, w as i64) as isize;
            let y = self.random_range(0, h as i64) as isize;
            if !blocks.iter().any(|block| block.x == x && block.y == y) {
                return Some(Vec2 { x, y });
            }
        }

        for y in 0..h {
            for x in 0..w {
                let (xi, yi) = (x as isize, y as isize);
                if !blocks.iter().any(|block| block.x == xi && block.y == yi) {
                    return Some(Vec2 { x: xi, y: yi });
                }
            }
        }

        None
    }
}
