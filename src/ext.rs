/// Extensions for booleans
pub trait BoolExt {
    /// Toggle the value
    fn toggle(&mut self);
}

impl BoolExt for bool {
    /// so `true` becomes `false` and vice versa
    #[inline]
    fn toggle(&mut self) {
        *self = !*self;
    }
}

/// Extensions for floats
pub trait FloatExt {
    /// Toggle the value
    fn limit(self, min: Self, max: Self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn limit(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

#[derive(Debug, Default)]
/// Tracks how many buttons are being pressed in specific directions
pub struct InputState {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
}

impl InputState {
    #[inline]
    /// Returns `-1`, `0` or `1` depending on whether `self.hor` is negative, zero or positive
    pub fn hor(&self) -> f32 {
        f32::from(self.right as i8 - self.left as i8).signum()
    }
    /// Returns `-1`, `0` or `1` depending on whether `self.ver` is negative, zero or positive
    #[inline]
    pub fn ver(&self) -> f32 {
        f32::from(self.down as i8 - self.up as i8).signum()
    }
}

#[derive(Debug, Default)]
pub struct MouseDown {
    pub left: bool,
    pub middle: bool,
    pub right: bool,
}

#[derive(Debug, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}
