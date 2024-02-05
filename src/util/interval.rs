#[derive(Debug, Clone, Copy)]
pub struct Interval {
    min: f64,
    max: f64,
}

impl Default for Interval {
    fn default() -> Self {
        Self::empty()
    }
}

impl Interval {
    #[must_use]
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    #[must_use]
    pub const fn universe() -> Self {
        Self {
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
        }
    }

    #[must_use]
    pub const fn from(min: f64) -> Self {
        Self {
            min,
            max: f64::INFINITY,
        }
    }

    #[must_use]
    pub const fn till(max: f64) -> Self {
        Self {
            min: f64::NEG_INFINITY,
            max,
        }
    }

    #[must_use]
    pub fn contains_inc(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    #[must_use]
    pub fn contains_ex(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }
}

#[test]
fn universe_contains_inc() {
    let inter = Interval::universe();

    assert!(inter.contains_inc(0.));
    assert!(inter.contains_inc(f64::INFINITY));
    assert!(inter.contains_inc(f64::NEG_INFINITY));
    assert!(inter.contains_inc(f64::MIN_POSITIVE));
    assert!(inter.contains_inc(f64::MAX));

    assert!(!inter.contains_inc(f64::NAN));
}
#[test]
fn empty_contains_inc() {
    let inter = Interval::empty();

    assert!(!inter.contains_inc(0.));
    assert!(!inter.contains_inc(f64::INFINITY));
    assert!(!inter.contains_inc(f64::NEG_INFINITY));
    assert!(!inter.contains_inc(f64::MIN_POSITIVE));
    assert!(!inter.contains_inc(f64::MAX));
    assert!(!inter.contains_inc(f64::NAN));
}
#[test]
fn range_contains_inc() {
    let inter = Interval::new(-10., 0.3);

    assert!(inter.contains_inc(-10.));
    assert!(inter.contains_inc(0.3));
    assert!(inter.contains_inc(0.));
    assert!(inter.contains_inc(f64::MIN_POSITIVE));

    assert!(!inter.contains_inc(-11.));
    assert!(!inter.contains_inc(0.301));
    assert!(!inter.contains_inc(f64::NEG_INFINITY));
    assert!(!inter.contains_inc(f64::INFINITY));
    assert!(!inter.contains_inc(f64::MAX));
    assert!(!inter.contains_inc(f64::NAN));
}

#[test]
fn universe_contains_ex() {
    let inter = Interval::universe();

    assert!(inter.contains_ex(0.));
    assert!(inter.contains_ex(f64::MIN_POSITIVE));
    assert!(inter.contains_ex(f64::MAX));

    assert!(!inter.contains_ex(f64::INFINITY));
    assert!(!inter.contains_ex(f64::NEG_INFINITY));
    assert!(!inter.contains_ex(f64::NAN));
}
#[test]
fn empty_contains_ex() {
    let inter = Interval::empty();

    assert!(!inter.contains_ex(0.));
    assert!(!inter.contains_ex(f64::INFINITY));
    assert!(!inter.contains_ex(f64::NEG_INFINITY));
    assert!(!inter.contains_ex(f64::MIN_POSITIVE));
    assert!(!inter.contains_ex(f64::MAX));
    assert!(!inter.contains_ex(f64::NAN));
}
#[test]
fn range_contains_ex() {
    let inter = Interval::new(-10., 0.3);

    assert!(inter.contains_ex(-9.99));
    assert!(inter.contains_ex(0.299));
    assert!(inter.contains_ex(0.));
    assert!(inter.contains_ex(f64::MIN_POSITIVE));

    assert!(!inter.contains_ex(-11.));
    assert!(!inter.contains_ex(0.301));
    assert!(!inter.contains_ex(-10.));
    assert!(!inter.contains_ex(0.3));
    assert!(!inter.contains_ex(f64::NEG_INFINITY));
    assert!(!inter.contains_ex(f64::INFINITY));
    assert!(!inter.contains_ex(f64::MAX));
    assert!(!inter.contains_ex(f64::NAN));
}
