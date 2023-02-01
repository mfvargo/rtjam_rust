//! Constant power left/right fader

use crate::utils::clip_float;
use std::fmt;

pub struct Fader {
    left: f32,
    right: f32,
}

impl Fader {
    pub fn new() -> Fader {
        let mut f = Fader {
            left: 1.0,
            right: 1.0,
        };
        f.set(0.0);
        f
    }

    pub fn set(&mut self, v: f32) -> () {
        let fade = clip_float(v);
        self.left = f32::sqrt(1.0 - fade);
        self.right = f32::sqrt(1.0 + fade);
    }

    pub fn left(&self) -> f32 {
        self.left
    }

    pub fn right(&self) -> f32 {
        self.right
    }
}

impl fmt::Display for Fader {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ left: {}, right: {} ]", self.left, self.right)
    }
}
#[cfg(test)]

mod test_fader {
    use super::*;

    #[test]
    fn build_and_use() {
        let mut fader = Fader::new();
        assert_eq!(fader.left(), 1.0);
        assert_eq!(fader.right(), 1.0);
        // Hard pan left
        fader.set(-1.0);
        assert_eq!(fader.left(), f32::sqrt(2.0));
        assert_eq!(fader.right(), 0.0);
        fader.set(1.0);
        assert_eq!(fader.right(), f32::sqrt(2.0));
        assert_eq!(fader.left(), 0.0);
    }
}
