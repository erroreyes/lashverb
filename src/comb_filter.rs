use crate::delay_buffer::DelayBuffer;

pub struct Comb {
    delay_buffer: DelayBuffer, 
    pub buffer_len: usize, 
    delay: usize, 
    feedback: f32, 
    filter_state: f32, 
    damp: f32, 
    damp_inv: f32, 
}

impl Comb {
    pub fn new(delay_length: usize) -> Self {
        Self {
            delay_buffer: DelayBuffer::new(delay_length), 
            buffer_len: delay_length, 
            delay: delay_length, 
            feedback: 0.5, 
            filter_state: 0.0, 
            damp: 0.5, 
            damp_inv: 0.5,
        }
    }

    pub fn set_delay(&mut self, x: usize) {
        self.delay = x;
    }

    pub fn set_damp(&mut self, value: f32) {
        self.damp = value;
        // the invert of the value
        self.damp_inv = 1.0 - value;
    }

    pub fn set_feedback(&mut self, value: f32) {
        self.feedback = value;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // get the current output form the delay buffer
        let output = self.delay_buffer.read();
        
        // calculate a new filter state by applying the inverse damp to the delay 
        // output, applying the damp value to the current filter state, and 
        // adding both
        self.filter_state = output * self.damp_inv + self.filter_state * self.damp;
        
        // process the feedback signal separately.
        let feedback_out = self.filter_state * self.feedback;

        // write the current input and the current filtered feedback signal to 
        // the delay buffer 
        self.delay_buffer.write(input + feedback_out);
        
        // get the delay length as percentage of the buffer length
        let d_len = self.buffer_len * self.delay / 100;
        
        // advance the buffer index up to buffer length
        self.delay_buffer.advance_to(d_len - 1);
        
        // return the current output from the delay buffer
        output
    }
}