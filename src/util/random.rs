use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct XorShift {
    state: u128,
}

impl Default for XorShift {
    /// Creates a new [`XorShift`] based on the current milliseconds since unix
    /// epoch. This is stored in a u128, so it'll be fine for a while. Important
    /// THIS IS NOT CRYPTOGRAPHICALLY SECURE, THIS JUST WORKS.
    ///
    /// # Panics
    ///
    /// Panics if it's the year 9.67 * 10 ^24 or if it's before unix epoch.
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time died, mate what fucking year is it :o")
            .as_millis();
        Self { state: now }
    }
}

impl XorShift {
    /// Creates a new [`XorShift`] seed. This is stored in a u128, so it'll be
    /// fine for a while. Important THIS IS NOT CRYPTOGRAPHICALLY SECURE, THIS
    /// JUST WORKS.
    pub const fn new(seed: u128) -> Self {
        Self { state: seed }
    }

    pub fn next_int(&mut self) -> u128 {
        self.state ^= self.state << 23;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 26;
        self.state
    }

    pub fn next_01(&mut self) -> f64 {
        let next = self.next_int();

        let next_bounded = next % u128::from(u32::MAX);
        let next_u32 = u32::try_from(next_bounded) //
            .expect("The u32 was bigger than u32, wtf");

        let output: f64 = f64::from(next_u32) / f64::from(u32::MAX);

        debug_assert!(output <= 1.);

        output
    }

    pub fn next_bound(&mut self, min: f64, max: f64) -> f64 {
        let diff = max - min;
        let next = self.next_01();

        min + diff * next
    }

    pub fn copy_reset(&mut self) -> Self {
        let self_state = self.state;
        let mut reset_state = self_state ^ self.next_int();
        reset_state ^= reset_state >> 13;
        reset_state ^= reset_state << 5;
        reset_state ^= reset_state >> 11;

        Self { state: reset_state }
    }
}
