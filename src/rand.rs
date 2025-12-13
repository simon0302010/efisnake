use uefi::runtime::get_time;

pub struct Rng {
    pub state: f64,
}

impl Rng {
    /// creates a new rng
    pub fn new() -> Self {
        let seed: f64 = match get_time() {
            Ok(time) => time.nanosecond() as f64,
            Err(_) => {
                234246782374.0 // not really random
            }
        };

        Self { state: seed }
    }

    // generates a random integer in the specified range
    pub fn random_range(&mut self, min: i64, max: i64) -> i64 {
        (self.random_float() * ((max - min + 1) + min) as f64) as i64
    }

    /// generates a random float from 0.0 to 1.0
    pub fn random_float(&mut self) -> f64 {
        let a = 6364136223846793005;
        let c = 1442695040888963407;
        let m = core::i64::MAX;

        self.state = ((a * self.state as i64 + c) % m) as f64;
        self.state / m as f64
    }
}
