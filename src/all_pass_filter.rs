use crate::delay_buffer::DelayBuffer;

pub struct AllPass {
    delay_buffer: DelayBuffer
}

impl AllPass {
    pub fn new(length: usize) -> Self {
        Self {
            delay_buffer: DelayBuffer::new(length)
        }
    }

    pub fn process(&mut self, input: f32, feedback: f32) -> f32 {
        let delayed = self.delay_buffer.read();
        self.delay_buffer.write(input + delayed * feedback);
        self.delay_buffer.advance();
        -input + delayed
    }
}