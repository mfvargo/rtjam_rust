//! BiQuad filter

#[derive(Copy, Clone, Debug)]
pub enum FilterType {
    LowPass,
    HighPass,
    Peaking,
    LowShelf,
    HighShelf,
    AllPass,
    BandPass,
    Notch,
}

pub struct BiQuadFilter {
    pub filter_type: FilterType,
    pub sample_rate: f64,
    pub cutoff_freq: f64,
    pub cut_boost: f64,
    pub q: f64,
    // coeffs used to run the biquad
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    // Intermediate values used to calc coeffs
    a: f64,
    omega: f64,
    cos_omega: f64,
    sin_omega: f64,
    alpha: f64,
    // Values for previous outputs used to calculate current output
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}
impl BiQuadFilter {
    pub fn new() -> BiQuadFilter {
        BiQuadFilter {
            filter_type: FilterType::LowPass,
            sample_rate: 48_000.0,
            cutoff_freq: 1.0,
            cut_boost: 1.0,
            q: 1.0,
            // coeffs used to run the biquad
            a0: 0.0,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            // Intermediate values used to calc coeffs
            a: 0.0,
            omega: 0.0,
            cos_omega: 0.0,
            sin_omega: 0.0,
            alpha: 0.0,
            // Values for previous outputs used to calculate current output
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
    pub fn get_type(&self) -> FilterType {
        self.filter_type
    }
    pub fn init(
        &mut self,
        filter_type: FilterType,
        cutoff: f64,
        boost: f64,
        q: f64,
        sample_rate: f64,
    ) -> () {
        self.sample_rate = sample_rate;
        self.filter_type = filter_type;
        self.cutoff_freq = cutoff;
        self.cut_boost = boost;
        self.q = q;
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
        match self.filter_type {
            FilterType::LowPass => {
                self.calc_intermediate(1.0, self.q);
                self.b0 = (1.0 - self.cos_omega) / 2.0;
                self.b1 = 1.0 - self.cos_omega;
                self.b2 = (1.0 - self.cos_omega) / 2.0;
                self.a0 = 1.0 + self.alpha;
                self.a1 = -2.0 * self.cos_omega;
                self.a2 = 1.0 - self.alpha;
            }
            FilterType::HighPass => {
                self.calc_intermediate(1.0, self.q);
                self.b0 = (1.0 + self.cos_omega) / 2.0;
                self.b1 = -1.0 * (1.0 + self.cos_omega);
                self.b2 = (1.0 + self.cos_omega) / 2.0;
                self.a0 = 1.0 + self.alpha;
                self.a1 = -2.0 * self.cos_omega;
                self.a2 = 1.0 - self.alpha;
            }
            FilterType::Peaking => {
                self.calc_intermediate(self.cut_boost, self.q);
                self.alpha = self.sin_omega / (2.0 * self.q * self.a); // special case: alpha has term for cut/boost (not in LPF/HPF)
                self.a0 = 1.0 + (self.alpha / self.a);
                self.a1 = -2.0 * self.cos_omega;
                self.a2 = 1.0 - (self.alpha / self.a);
                self.b0 = 1.0 + (self.alpha * self.a); // * gainlinear
                self.b1 = -2.0 * self.cos_omega; // * gainlinear
                self.b2 = 1.0 - (self.alpha * self.a); // * gainlinear
            }
            FilterType::LowShelf => {
                self.calc_intermediate(self.cut_boost, self.q);
                self.alpha = (self.sin_omega / 2.0)
                    * f64::sqrt((self.a + (1.0 / self.a)) * ((1.0 / self.q) - 1.0) + 2.0); // special case for shelving filter
                self.b0 = self.a
                    * ((self.a + 1.0) - (self.a - 1.0) * self.cos_omega
                        + (2.0 * f64::sqrt(self.a) * self.alpha));
                self.b1 = 2.0 * self.a * ((self.a - 1.0) - (self.a + 1.0) * self.cos_omega);
                self.b2 = self.a
                    * ((self.a + 1.0)
                        - (self.a - 1.0) * self.cos_omega
                        - (2.0 * f64::sqrt(self.a) * self.alpha));
                self.a0 = (self.a + 1.0)
                    + (self.a - 1.0) * self.cos_omega
                    + (2.0 * f64::sqrt(self.a) * self.alpha);
                self.a1 = -2.0 * ((self.a - 1.0) + (self.a + 1.0) * self.cos_omega);
                self.a2 = (self.a + 1.0) + (self.a - 1.0) * self.cos_omega
                    - (2.0 * f64::sqrt(self.a) * self.alpha);
            }
            FilterType::HighShelf => {
                self.calc_intermediate(self.cut_boost, self.q);
                self.alpha = (self.sin_omega / 2.0)
                    * f64::sqrt((self.a + (1.0 / self.a)) * ((1.0 / self.q) - 1.0) + 2.0); // special case for shelving filter
                self.b0 = self.a
                    * ((self.a + 1.0)
                        + (self.a - 1.0) * self.cos_omega
                        + (2.0 * f64::sqrt(self.a) * self.alpha));
                self.b1 = -2.0 * self.a * ((self.a - 1.0) + (self.a + 1.0) * self.cos_omega);
                self.b2 = self.a
                    * ((self.a + 1.0) + (self.a - 1.0) * self.cos_omega
                        - (2.0 * f64::sqrt(self.a) * self.alpha));
                self.a0 = (self.a + 1.0) - (self.a - 1.0) * self.cos_omega
                    + (2.0 * f64::sqrt(self.a) * self.alpha);
                self.a1 = 2.0 * ((self.a - 1.0) - (self.a + 1.0) * self.cos_omega);
                self.a2 = (self.a + 1.0)
                    - (self.a - 1.0) * self.cos_omega
                    - (2.0 * f64::sqrt(self.a) * self.alpha);
            }
            FilterType::AllPass => {
                self.calc_intermediate(self.cut_boost, self.q);
                self.alpha = self.sin_omega / (2.0 * self.q * self.a); // special case: alpha has term for cut/boost (not in LPF/HPF)
                self.b0 = 1.0 - self.alpha;
                self.b1 = -2.0 * self.cos_omega;
                self.b2 = 1.0 + self.alpha;
                self.a0 = 1.0 + self.alpha;
                self.a1 = -2.0 * self.cos_omega;
                self.a2 = 1.0 - self.alpha;
            }
            FilterType::Notch => {
                self.calc_intermediate(self.cut_boost, self.q);
                self.alpha = self.sin_omega / (2.0 * self.q * self.a); // special case: alpha has term for cut/boost (not in LPF/HPF)
                self.b0 = 1.0;
                self.b1 = -2.0 * self.cos_omega;
                self.b2 = 1.0;
                self.a0 = 1.0 + self.alpha;
                self.a1 = -2.0 * self.cos_omega;
                self.a2 = 1.0 - self.alpha;
            }
            FilterType::BandPass => {
                self.calc_intermediate(self.cut_boost, self.q);
                self.alpha = self.sin_omega / (2.0 * self.q * self.a); // special case: alpha has term for cut/boost (not in LPF/HPF)
                self.b0 = self.alpha; // *m_q;
                self.b1 = 0.0;
                self.b2 = -1.0 * self.alpha; // *m_q;
                self.a0 = 1.0 + self.alpha;
                self.a1 = -2.0 * self.cos_omega;
                self.a2 = 1.0 - self.alpha;
            }
        }
        self.normalize_coeffs();
    }
    fn calc_intermediate(&mut self, cut_boost: f64, q: f64) {
        self.a = f64::powf(10.0, cut_boost / 40.0);
        self.omega = 2.0 * std::f64::consts::PI * (self.cutoff_freq / self.sample_rate);
        self.cos_omega = f64::cos(self.omega);
        self.sin_omega = f64::sin(self.omega);
        self.alpha = self.sin_omega / (2.0 * q);
    }
    fn normalize_coeffs(&mut self) -> () {
        self.b2 /= self.a0;
        self.b1 /= self.a0;
        self.b0 /= self.a0;
        self.a2 /= self.a0;
        self.a1 /= self.a0;
        self.a0 = 1.0;
    }

    pub fn get_sample(&mut self, input: f32) -> f32 {
        let value: f64 = self.b0 * input as f64 + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input as f64;
        self.y2 = self.y1;
        self.y1 = value;
        value as f32
    }
}

#[cfg(test)]
mod test_biquad {
    use super::*;

    #[test]
    fn can_build() {
        let filter = BiQuadFilter::new();
        assert_eq!(filter.sample_rate, 48_000.0);
    }
    #[test]
    fn can_init() {
        let mut filter = BiQuadFilter::new();
        filter.init(FilterType::LowPass, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::HighPass, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::Peaking, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::LowShelf, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::HighShelf, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::AllPass, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::BandPass, 400.0, 1.0, 2.0, 48_000.0);
        filter.init(FilterType::Notch, 400.0, 1.0, 2.0, 48_000.0);
        assert_eq!(filter.cut_boost, 1.0);
    }
    #[test]
    fn can_run() {
        let samps = vec![1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0];
        let mut res: Vec<f32> = vec![];
        let mut filter = BiQuadFilter::new();
        filter.init(FilterType::LowPass, 400.0, 1.0, 2.0, 48_000.0);
        for samp in samps {
            res.push(filter.get_sample(samp));
        }
        println!("result: {:?}", res);
    }
}
