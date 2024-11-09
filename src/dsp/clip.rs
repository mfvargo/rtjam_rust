//! signal clipper with 4 types,  Hard, Soft, Asymmetric and Even

pub enum ClipType {
    Hard,
    Soft,
    Asymmetric,
    Even,
    Exp,
}

/// output will be modified based on the clip type
/// - Hard => samples are hard clipped at +- 0.5
/// - Soft => softer clip using x / (1 + |x|)
/// - Asymmetric => postive samples same as soft, neg samples x / (1 + 3|x|), Creates DC offset!
/// - Even => |x| / (1 + |x|), Creates massive DC offset as all values are positive
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
        ClipType::Exp => {
            if sample_in > 0.0 {
                1.0 - f32::exp(-sample_in)
            } else {
                -1.0 + f32::exp(sample_in)
            }
        },
    }
}
