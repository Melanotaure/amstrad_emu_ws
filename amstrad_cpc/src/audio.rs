use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use ringbuf::{Consumer, HeapRb, Producer, SharedRb};
use std::{mem::MaybeUninit, sync::Arc};

type SampleRb = SharedRb<f32, Vec<MaybeUninit<f32>>>;

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: Consumer<f32, Arc<SampleRb>>,
) -> cpal::Stream
where
    T: SizedSample + FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                // The audio thread consumes the Ring Buffer
                for sample in data.iter_mut() {
                    *sample = T::from_sample(consumer.pop().unwrap_or(0.0));
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )
        .unwrap()
}

pub struct AudioSystem {
    pub producer: Producer<f32, Arc<SampleRb>>,
    _stream: cpal::Stream,
    pub sample_rate: f32,
    /// Number of device channels: each emulated sample must be pushed once per channel
    pub channels: usize,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioSystem {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No audio device found");

        let supported = device.default_output_config().unwrap();
        let sample_format = supported.sample_format();
        let sample_rate = supported.sample_rate().0 as f32;
        let channels = supported.channels() as usize;
        let config: cpal::StreamConfig = supported.into();

        // Buffer of about 50 ms to absorb the jitter of winit
        let latency_frames = (sample_rate as f32 * 0.05) as usize;
        let ring_buffer = HeapRb::<f32>::new(latency_frames * channels);
        let (producer, consumer) = ring_buffer.split();

        let stream = match sample_format {
            SampleFormat::I16 => build_stream::<i16>(&device, &config, consumer),
            SampleFormat::U16 => build_stream::<u16>(&device, &config, consumer),
            SampleFormat::I32 => build_stream::<i32>(&device, &config, consumer),
            SampleFormat::F32 => build_stream::<f32>(&device, &config, consumer),
            SampleFormat::F64 => build_stream::<f64>(&device, &config, consumer),
            other => panic!("Unsupported audio sample format: {other:?}"),
        };

        stream.play().unwrap();

        println!(
            "Audio initialized : {} Hz, {} channels",
            sample_rate, channels
        );

        Self {
            producer,
            _stream: stream,
            sample_rate,
            channels,
        }
    }
}
