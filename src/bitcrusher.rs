pub struct BitCrusher {
    bit_rate: f32, 
    sample_rate: f32, 
    prev_sample: f32, 
    noise_level: f32, 
}

impl BitCrusher {
    pub fn new() -> Self {
        Self {
            bit_rate: 6.0, 
            sample_rate: 1.0, 
            prev_sample: 0.0, 
            noise_level: 0.0, 
        }
    }

    pub fn set_bit_rate(&mut self, x: f32) {
        self.bit_rate = x;
    }

    pub fn set_sample_rate(&mut self, x: f32) {
        self.sample_rate = x;
    }

    pub fn set_noise_level(&mut self, x: f32) {
        self.noise_level = x;
    }

    pub fn process(&mut self, x: f32, 
                buffer_index: usize, 
                buffer_size: usize) -> f32 {
        let bit = 2.0_f32.powf(self.bit_rate);
        let sample_scaled = bit * (0.5 * x + 0.5);
        let sample_rounded = sample_scaled.floor();
        let mut sample_rescaled = 2.0 * (sample_rounded / bit) - 1.0;
        
        let idx = buffer_index % self.sample_rate as usize;

        if self.sample_rate > 1.0 {
            if idx > 0 && idx < buffer_size {
                sample_rescaled = self.prev_sample;
            } 
        } else {
            self.prev_sample = sample_rescaled;
        }
        sample_rescaled
    }
}