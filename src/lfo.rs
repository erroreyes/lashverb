use std::f32::consts::PI;

pub struct Lfo {
    /// The current sample rate
    sample_rate: usize,
    /// LFO shape
    shape: Shape,
    /// Amplitude from 0.0 to 1.0 
    amp: f32, 
    /// Speed in hertz
    speed: f32,
    /// Phase
    phase: f32, 
}

pub enum Shape {
    Sine, 
    Triangle, 
    Sawtooth, 
    Square, 
    Random
}

impl Lfo {
    pub fn new(sr: usize) -> Self {
        Self {
            sample_rate: sr,
            shape: Shape::Sine, 
            amp: 100.0,
            speed: 5000.0, 
            phase: 0.0,
        }
    }

    pub fn set_amp(&mut self, x: f32) {
        self.amp = x;
    }

    pub fn set_speed(&mut self, x: f32) {
        self.speed = x;
    }

    pub fn set_shape(&mut self, x: Shape) {
        self.shape = x;
    }

    pub fn output(&mut self) -> f32 {
        self.phase += 2.0 * PI * self.speed / self.sample_rate as f32;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        match self.shape {
            Shape::Sine => { self.amp * self.phase.sin() },
            Shape::Triangle => { 0.0 }, 
            Shape::Sawtooth => { 0.0 }, 
            Shape::Square => { 0.0 }, 
            Shape::Random => { 0.0 },
        }
    }
}