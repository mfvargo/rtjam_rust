use crate::common::{get_micro_time, stream_time_stat::MicroTimer};


pub struct Metronome {
    beat: u8,
    duration: MicroTimer,
    tempo: u128,
}

impl Metronome {
    pub fn new() -> Metronome {
        Metronome {
            beat: 0,
            tempo: 120,
            duration: MicroTimer::new(get_micro_time(), 1_000_000 * 60 / 120), // 120BPM in microseconds
        }
    }
    pub fn set_tempo(&mut self, now_time: u128,  tempo: u128) -> () {
        if tempo > 0 {
            self.tempo = tempo;
            self.duration = MicroTimer::new(now_time, 1_000_000 * 60 / tempo);
        }
    }
    pub fn get_tempo(&self) -> u128 {
        self.tempo
    }
    pub fn get_beat_interval(&self) -> u128 {
        self.duration.get_interval()
    }
    pub fn get_beat(&mut self, now_time: u128)  -> u8 {
        if self.duration.expired(now_time) {
            self.duration.advance(self.duration.get_interval());
            self.beat += 1;
        }
        self.beat = self.beat % 4;
        self.beat
    }
    pub fn reset_time(&mut self, now_time: u128) -> () {
        self.duration.reset(now_time);
    }
}

#[cfg(test)]
mod test_metronome {
    use super::*;

    #[test]
    fn test_beat() {
        let now = 1000;
        let mut met = Metronome::new();
        met.set_tempo(now, 120);
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
