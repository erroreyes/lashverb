// A simple delay buffer implementation

pub struct DelayBuffer {
    buffer: Vec<f32>,
    index: usize
}

impl DelayBuffer {
    /// Creates a new instance of the buffer.
    pub fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length],
            index: 0
        }
    }

    /// Reads audio from the buffer.
    pub fn read(&mut self) -> f32 {
        self.buffer[self.index]
    }

    /// Writes audio to the buffer. 
    /// Any of the `advance...()` functions should be called after this. 
    pub fn write(&mut self, input: f32) {
        self.buffer[self.index] = input;
        // self.advance();
    }

    /// Advances the buffer position up to `buffer.len() - 1`.
    pub fn advance(&mut self) {
        self.advance_to(self.buffer.len() - 1);
    }

    /// Advances the buffer position up to the `to`.
    pub fn advance_to(&mut self, to: usize) {
        if self.index < to {
            self.index += 1;
        } else {
            self.index = 0;
        }
    }
}
