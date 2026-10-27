use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapRb, Producer, SharedRb};
use std::{mem::MaybeUninit, sync::Arc};

pub struct AudioSystem {
    pub producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
    _stream: cpal::Stream,
    pub sample_rate: f32,
}

impl AudioSystem {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No audio device found");

        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        // Buffer d'environ 50 ms pour absorber la gigue (jitter) de winit
        let latency_frames = (sample_rate as f32 * 0.05) as usize;
        let ring_buffer = HeapRb::<f32>::new(latency_frames * channels);
        let (producer, mut consumer) = ring_buffer.split();

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Le thread audio consomme le Ring Buffer
                    for sample in data.iter_mut() {
                        *sample = consumer.pop().unwrap_or(0.0);
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        println!(
            "Audio initialized : {} Hz, {} channels",
            sample_rate, channels
        );

        Self {
            producer,
            _stream: stream,
            sample_rate,
        }
    }
}
