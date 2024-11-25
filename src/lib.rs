
use nih_plug::prelude::*;
use reverb::Reverb;
use std::sync::Arc;

mod all_pass_filter;
mod comb_filter;
mod delay_buffer;
mod reverb;
mod lfo;
mod bitcrusher;

struct Lashverb {
    params: Arc<LashverbParams>,
    reverb: Reverb,
}

#[derive(Params)]
struct LashverbParams {
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "size"]
    pub size: IntParam,
    #[id = "damp"]
    pub damp: FloatParam,
    #[id = "width"]
    pub width: FloatParam,
    #[id = "bit_rate"]
    pub bit_rate: FloatParam,
    #[id = "wet"]
    pub wet: FloatParam,
    #[id = "dry"]
    pub dry: FloatParam,
}

impl Default for Lashverb {
    fn default() -> Self {
        Self {
            params: Arc::new(LashverbParams::default()),
            reverb: Reverb::new(44100),
        }
    }
}

impl Default for LashverbParams {
    fn default() -> Self {
        Self {
            // --------------------------------------------------------------------------------
            // Decay
            decay: FloatParam::new(
                "Decay",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage())
            ,
            
            // --------------------------------------------------------------------------------
            // Size
            size: IntParam::new(
                "Size", 
                50, 
                IntRange::Linear { min: 50, max: 100 }
            )
            .with_unit("%")
            .with_smoother(SmoothingStyle::Linear(10.0))
            ,

            // --------------------------------------------------------------------------------
            // Damp
            damp: FloatParam::new(
                "Damp",
                0.3,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage())
            ,
            
            // --------------------------------------------------------------------------------
            // Width
            width: FloatParam::new(
                "Width",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage())
            ,
            
            // --------------------------------------------------------------------------------
            // Bit Rate
            bit_rate: FloatParam::new(
                "Bit Rate",
                1.0,
                FloatRange::Linear {
                    min: 4.0,
                    max: 16.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" bit")
            .with_value_to_string(formatters::v2s_f32_rounded(1))
            ,

            // --------------------------------------------------------------------------------
            // Wet 
            wet: FloatParam::new(
                "Wet",
                util::db_to_gain(-9.0),
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db())
            ,
            
            // --------------------------------------------------------------------------------
            // Dry
            dry: FloatParam::new(
                "Dry",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db())
            ,
            
        }
    }
}

impl Plugin for Lashverb {
    const NAME: &'static str = "Lashverb";
    const VENDOR: &'static str = "LASHLIGHT";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "your@email.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = _buffer_config.sample_rate as usize;
        self.reverb = Reverb::new(sample_rate);
        true
    }

    fn reset(&mut self) {
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        /*
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample *= gain;
            }
        }
        */

        let num_samples = buffer.samples();
        let out = buffer.as_slice();

        for i in 0..num_samples {
            self.reverb.decay(self.params.decay.value());
            self.reverb.size(self.params.size.value() as usize);
            self.reverb.damp(self.params.damp.value());
            self.reverb.width(self.params.width.value());
            self.reverb.wet(self.params.wet.value());
            self.reverb.dry(self.params.dry.value());
            // For the bit crusher
            self.reverb.set_idx(i);
            self.reverb.set_size(num_samples);
            self.reverb.set_bit_rate(self.params.bit_rate.value());

            let (in_left, in_right) = (out[0][i], out[1][i]);
            let (out_left, out_right) = self.reverb.process((in_left, in_right));
            out[0][i] = out_left;
            out[1][i] = out_right;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Lashverb {
    const CLAP_ID: &'static str = "com.lashlight.lashverb";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple Freeverb-based reverb that aims at not sounding right");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect, 
        ClapFeature::Stereo, 
        ClapFeature::Reverb
    ];
}

impl Vst3Plugin for Lashverb {
    const VST3_CLASS_ID: [u8; 16] = *b"lashverblashverb";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nih_export_clap!(Lashverb);
nih_export_vst3!(Lashverb);
