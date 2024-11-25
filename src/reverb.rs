use crate::comb_filter::Comb;
use crate::all_pass_filter::AllPass;
use crate::lfo::Lfo;
use crate::bitcrusher::BitCrusher;

// Add this to any of the constants below to create stereo difference, for example:
//     use COMB_TUNING_1 on the left stereo channel
//     use COMB_TUNING_1 + STEREO_OFFSET on the right
// NOTE: original value is 23, so change back if any other values are shit
const OFFSET: usize = 34;

// NOTE: see the original values if these are shit
const COMB_TUN_1: usize = 1116;
const COMB_TUN_2: usize = COMB_TUN_1 + 72;
const COMB_TUN_3: usize = COMB_TUN_1 + 89;
const COMB_TUN_4: usize = COMB_TUN_1 + 105;
const COMB_TUN_5: usize = COMB_TUN_1 + 122;
const COMB_TUN_6: usize = COMB_TUN_1 + 139;
const COMB_TUN_7: usize = COMB_TUN_1 + 157;
const COMB_TUN_8: usize = COMB_TUN_1 + 174;

const AP_TUN_1: usize = 225;
const AP_TUN_2: usize = 331;
const AP_TUN_3: usize = 431;
const AP_TUN_4: usize = 547;

pub struct Reverb {
    // 8 comb filters for left and right channels
    combs: [(Comb, Comb); 8], 
    // 4 all pass filters for left and right channels
    allpasses: [(AllPass, AllPass); 4], 
    // 2 reverb gains, one for the main reverb output, and the other
    // for the additional crossfeed
    wet_gains: (f32, f32),
    wet: f32, 
    width: f32, 
    dry: f32,
    input_gain: f32, 
    damp: f32, 
    decay: f32, 
    size: usize, 
    lfo: Lfo,
    freeze: bool, 
    bitcrusher: BitCrusher, 
    buffer_data: BufferData,
    counter: f32,
    fadeout_factor: f32,
}

pub struct BufferData {
    buffer_idx: usize,
    buffer_size: usize
}

fn calc_len(length: usize, sample_rate: usize) -> usize {
    length * sample_rate / 44100
}

impl Reverb {
    ///
    /// Constructor
    /// * `sample_rate` - the current sample rate
    pub fn new(sample_rate: usize) -> Self {
        let cbs = [
            (
                Comb::new(calc_len(COMB_TUN_1, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_1 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_2, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_2 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_3, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_3 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_4, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_4 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_5, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_5 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_6, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_6 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_7, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_7 + OFFSET, sample_rate))
            ),
            (
                Comb::new(calc_len(COMB_TUN_8, sample_rate)), 
                Comb::new(calc_len(COMB_TUN_8 + OFFSET, sample_rate))
            )
        ];

        let aps = [
            (
                AllPass::new(calc_len(AP_TUN_1, sample_rate)), 
                AllPass::new(calc_len(AP_TUN_1 + OFFSET, sample_rate))
            ),
            (
                AllPass::new(calc_len(AP_TUN_2, sample_rate)), 
                AllPass::new(calc_len(AP_TUN_2 + OFFSET, sample_rate))
            ),
            (
                AllPass::new(calc_len(AP_TUN_3, sample_rate)), 
                AllPass::new(calc_len(AP_TUN_3 + OFFSET, sample_rate))
            ),
            (
                AllPass::new(calc_len(AP_TUN_4, sample_rate)), 
                AllPass::new(calc_len(AP_TUN_4 + OFFSET, sample_rate))
            )
        ];

        let mut reverb = Reverb {
            combs: cbs,
            allpasses: aps, 
            wet_gains: (0.0, 0.0), 
            wet: 0.8,
            width: 1.0, 
            dry: 0.2, 
            input_gain: 1.0, 
            damp: 0.5, 
            decay: 0.67, 
            size: COMB_TUN_1 + OFFSET,
            lfo: Lfo::new(sample_rate), 
            freeze: false, 
            bitcrusher: BitCrusher::new(), 
            buffer_data: BufferData::new(),
            counter: 0.0,
            fadeout_factor: 0.0,
        };

        // Init the reverb and return
        reverb.update_wet_gains();
        reverb.update();
        reverb
    }

    /// Processes the `input`, returns it with reverb applied.
    pub fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let in_sum = (input.0 + input.1) * 0.015 * self.input_gain;
        let mut out = (0.0, 0.0);

        self.lfo.set_speed(44100.0);
        self.lfo.set_amp(20.0);

        // Apply comb filters
        for c in self.combs.iter_mut() {
            // Apply the LFO to the delay
            let mut mod_delay = self.size as f32 + self.lfo.output();
            
            // Make sure the modulated delay does not passed the length of the buffer
            mod_delay = mod_delay.min(c.0.buffer_len as f32 - 1.0);

            c.0.set_delay(mod_delay as usize);
            c.1.set_delay(mod_delay as usize);

            out.0 += c.0.process(input.0 * 0.015 + in_sum);
            out.1 += c.1.process(input.1 * 0.015 + in_sum);
        }

        // Apply allpass filters
        for a in self.allpasses.iter_mut() {
            out.0 = a.0.process(out.0, 0.5);
            out.1 = a.1.process(out.1, 0.5);
        }

        // Apply bit crush
        out.0 = self.bitcrusher.process(
            out.0, 
            self.buffer_data.buffer_idx, 
            self.buffer_data.buffer_size
        );
        out.1 = self.bitcrusher.process(
            out.1, 
            self.buffer_data.buffer_idx, 
            self.buffer_data.buffer_size
        );

        // Increase the counter if the input signal falls below 0.002;
        // reset the counter if input signal is received
        self.counter = if (input.0 < 0.0002) && (input.1 < 0.0002) {
            self.counter + 0.001
        } else {
            0.0
        };

        // Apply fade out gradually, and make sure the fadeout factor
        // never becomes negative.
        self.fadeout_factor = (1.0 - self.counter / 500.0).max(0.0);

        // Get reverb and crossfeed
        let (mut rev_0, mut rev_1) = (
            // main out               + crossfeed out             * fadeout       
            (out.0 * self.wet_gains.0 + out.1 * self.wet_gains.1) * self.fadeout_factor,
            (out.1 * self.wet_gains.0 + out.0 * self.wet_gains.1) * self.fadeout_factor,
        );

        // Denormalize!
        if !rev_0.is_normal() {
            rev_0 = 0.0;
        }

        if !rev_1.is_normal() {
            rev_1 = 0.0;
        }

        // Return reverb and dry signal
        (
            rev_0 + input.0 * self.dry, 
            rev_1 + input.1 * self.dry,
        )
    }

    /// Update
    fn update(&mut self) {
        let (feedback, damp) = if self.freeze {
            (1.0, 0.0)
        } else {
            (self.decay, self.damp)
        };

        for comb in self.combs.iter_mut() {
            comb.0.set_feedback(feedback);
            comb.0.set_damp(damp);
            comb.1.set_feedback(feedback);
            comb.1.set_damp(damp);
        }
    }

    /// Sets the dry signal level
    pub fn dry(&mut self, dry: f32) {
        self.dry = dry;
    }

    /// Sets the wet signal level
    pub fn wet(&mut self, wet: f32) {
        self.wet = wet * 1.0;
        self.update_wet_gains();
    }

    /// Updates the level of the wet and crossfeed signals 
    fn update_wet_gains(&mut self) {
        self.wet_gains = (
            self.wet * (self.width / 2.0 + 0.5), 
            self.wet * ((1.0 - self.width) / 2.0),
        )
    }

    /// Sets the stereo width
    pub fn width(&mut self, width: f32) {
        self.width = width;
        self.update();
    }

    /// Sets the reverb damping
    pub fn damp(&mut self, damp: f32) {
        self.damp = damp * 0.4;
        self.update();
    }

    /// Sets the decay of the reverb
    pub fn decay(&mut self, decay: f32) {
        self.decay = decay * 0.27 + 0.7;
        self.update();
    }

    /// Sets the size or the delay time
    pub fn size(&mut self, size: usize) {
        self.size = size;
        self.update();
    }

    #[warn(dead_code)]
    pub fn freeze(&mut self, is_freeze: bool) {
        self.freeze = is_freeze;
        self.update();
    }

    /// Sets the current buffer ID
    pub fn set_idx(&mut self, idx: usize) {
        self.buffer_data.buffer_idx = idx;
    }

    /// Sets the current buffer size
    pub fn set_size(&mut self, size: usize) {
        self.buffer_data.buffer_size = size;
    }

    /// Sets the bit rate of the bit crusher
    pub fn set_bit_rate(&mut self, bit_rate: f32) {
        self.bitcrusher.set_bit_rate(bit_rate);
    }
}

impl BufferData {
    pub fn new() -> Self {
        BufferData {
            buffer_idx: 0, 
            buffer_size: 0,
        }
    }
}