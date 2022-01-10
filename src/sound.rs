use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};

/// A simple abstraction used to play a beeping noise (sine wave).
///
/// Must manually play or pause the noise.
pub struct Beeper {
    stream: cpal::Stream,
}

impl Beeper {
    /// Creates a new paused beeper.
    pub fn new() -> anyhow::Result<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .expect("no audio device found");

        let mut supported_configs_range = device.supported_output_configs()?;
        let config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate();

        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into())?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into())?,
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into())?,
        };

        stream.pause()?;

        Ok(Beeper { stream })
    }

    /// Starts or resumes playback of an annoying beeping noise.
    pub fn play(&self) {
        self.stream.play().ok();
    }

    /// Stops playback.
    pub fn pause(&self) {
        self.stream.pause().ok();
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Result<Stream, anyhow::Error>
    where
        T: cpal::Sample,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        // Produce a sinusoid of maximum amplitude.
        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                Self::write_data(data, channels, &mut next_value)
            },
            err_fn,
        )?;

        return Ok(stream);
    }

    fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
    where
        T: cpal::Sample,
    {
        for frame in output.chunks_mut(channels) {
            let value: T = cpal::Sample::from::<f32>(&next_sample());
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}
