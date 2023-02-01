//! Constant power left/right fader

pub struct Fader {
    left: f32,
    right: f32,
}

impl Fader {
    pub fn new() -> Fader {
        Fader {
            left: 1.0,
            right: 1.0,
        }
    }

    pub fn set(&mut self, v: f32) -> () {
        let mut fade = v;
        if fade > 1.0 {
            fade = 1.0;
        }
        if fade < -1.0 {
            fade = -1.0;
        }
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
