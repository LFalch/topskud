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