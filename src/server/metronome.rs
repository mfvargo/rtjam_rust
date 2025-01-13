
pub struct Metronome {
    beat: u8,
    duration: u128,
    tempo: u128,
}

impl Metronome {
    pub fn new() -> Metronome {
        Metronome {
            beat: 0,
            tempo: 120,
            duration: 1_000_000 * 60 / 120,
        }
    }
    pub fn set_tempo(&mut self, tempo: u128) -> () {
        if tempo > 0 {
            self.tempo = tempo;
            self.duration = 1_000_000 * 60 / tempo;
        }
    }
    pub fn get_tempo(&self) -> u128 {
        self.tempo
    }
    pub fn get_beat_interval(&self) -> u128 {
        self.duration
    }
    pub fn get_beat(&mut self, now_time: u128)  -> u8 {
        self.beat = (now_time / self.duration % 4) as u8;
        self.beat
    }
}

#[cfg(test)]
mod test_metronome {
    use super::*;

    #[test]
    fn test_beat() {
        let now = 2_000_000;
        let mut met = Metronome::new();
        met.set_tempo(120);
        assert_eq!(met.get_beat_interval(), 1_000_000 / 2); // 120BPM = 500_000 microseconds
        assert_eq!(met.get_beat(now), 0);
        assert_eq!(met.get_beat(now + 499_999), 0);
        assert_eq!(met.get_beat(now + 500_001), 1);
        assert_eq!(met.get_beat(now + 500_001), 1);
        assert_eq!(met.get_beat(now + 1_000_001), 2);
        assert_eq!(met.get_beat(now + 1_000_001), 2);
        assert_eq!(met.get_beat(now + 1_500_001), 3);
        assert_eq!(met.get_beat(now + 2_000_001), 0);
        assert_eq!(met.get_beat(now + 2_500_001), 1);
    }
}
