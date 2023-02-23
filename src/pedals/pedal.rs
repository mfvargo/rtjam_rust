use super::controls::PedalSetting;

pub enum Algorithm {
    Filter,
    Envelope,
}

pub struct Pedal {
    settings: Vec<PedalSetting>,
    algorithm: Algorithm,
}
