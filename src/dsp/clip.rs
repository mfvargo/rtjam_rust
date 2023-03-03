pub enum ClipType {
    Hard,
    Soft,
    Asymmetric,
    Even,
}

pub fn clip_sample(ctype: &ClipType, sample_in: f32) -> f32 {
    match ctype {
        ClipType::Hard => sample_in.clamp(-0.5, 0.5),
        ClipType::Soft => sample_in / (1.0 + sample_in.abs()),
        ClipType::Asymmetric => {
            if sample_in > 0.0 {
                sample_in / (1.0 + sample_in.abs())
            } else {
                sample_in / (1.0 + (3.0 * sample_in).abs())
            }
        }
        ClipType::Even => sample_in.abs() / (1.0 + sample_in.abs()),
    }
}
