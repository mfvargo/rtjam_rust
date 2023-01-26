use crate::stream_time_stat::StreamTimeStat;

pub struct JitterBuffer {
    buffer: Vec<f32>,
    depth_stats: StreamTimeStat,
}

impl JitterBuffer {
    pub fn build() -> JitterBuffer {
        JitterBuffer {
            buffer: Vec::<f32>::new(),
            depth_stats: StreamTimeStat::build(50),
        }
    }
    pub fn length(&self) -> usize {
        self.buffer.len()
    }
    pub fn avg_depth(&self) -> f64 {
        self.depth_stats.get_mean()
    }
    pub fn put(&mut self, samples: &mut Vec<f32>) -> () {
        // This function should put in some data
        self.buffer.append(samples);
    }
    pub fn get(&mut self, count: usize) -> Vec<f32> {
        // It should get some data off the buffer

        self.depth_stats.add_sample(self.buffer.len() as f64); // Gather depth stats

        if self.buffer.len() == 0 {
            // No data in the buffer
            return vec![0.0; count];
        }
        if count > self.buffer.len() {
            // This is an underrun
            let remainder = count - self.buffer.len();
            // get the partial data
            let mut rval: Vec<f32> = self.buffer.drain(..).collect();
            // fill zeros on the end
            rval.append(&mut vec![0.0; remainder]);
            return rval;
        }
        self.buffer.drain(..count).collect()
    }
}

#[cfg(test)]
mod test_jitter_buffer {
    use super::*;

    #[test]
    fn build() {
        // you should be able to build a jitter buffer
        let buf = JitterBuffer::build();
        assert_eq!(buf.length(), 0);
    }
    #[test]
    fn avg_depth() {
        // It should tell you it's avg depth
        let buf = JitterBuffer::build();
        assert_eq!(buf.avg_depth(), 0.0);
    }
    #[test]
    fn put() {
        // It should have an append
        let mut buf = JitterBuffer::build();
        let mut samples: Vec<f32> = vec![0.2, 0.3, 0.4];
        buf.put(&mut samples);
        assert_eq!(buf.length(), 3);
    }
    #[test]
    fn get_normal() {
        // It should have a get function
        let mut buf = JitterBuffer::build();
        let mut samples: Vec<f32> = vec![0.2, 0.3, 0.4];
        buf.put(&mut samples);
        assert_eq!(buf.length(), 3);
        let res = buf.get(2);
        assert_eq!(res.len(), 2);
    }
    #[test]
    fn get_partial() {
        let mut buf = JitterBuffer::build();
        let mut samples: Vec<f32> = vec![0.2, 0.3, 0.4];
        buf.put(&mut samples);
        assert_eq!(buf.length(), 3);
        let res = buf.get(4);
        assert_eq!(res.len(), 4);
        assert_eq!(res, vec![0.2, 0.3, 0.4, 0.0]);
    }
    #[test]
    fn get_from_empty() {
        let mut buf = JitterBuffer::build();
        let res = buf.get(4);
        assert_eq!(res.len(), 4);
        assert_eq!(res, vec![0.0; 4]);
    }
}
