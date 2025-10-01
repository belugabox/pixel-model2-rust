//! Système audio SCSP (Saturn Custom Sound Processor) pour Model 2

use anyhow::Result;
use cpal::{traits::{HostTrait, DeviceTrait}, Stream, StreamConfig};

/// Émulateur du processeur sonore SCSP
pub struct ScspAudio {
    sample_rate: u32,
    channels: u16,
    _stream: Stream,
    pub volume: f32,
}

impl ScspAudio {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| anyhow::anyhow!("Aucun périphérique audio disponible"))?;
        
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        
        let stream_config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        
        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Génération audio simple
                for sample in data.iter_mut() {
                    *sample = 0.0; // Silence pour l'instant
                }
            },
            move |err| eprintln!("Erreur audio: {}", err),
            None,
        )?;
        
        Ok(Self {
            sample_rate,
            channels,
            _stream: stream,
            volume: 1.0,
        })
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
}

impl Default for ScspAudio {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Impossible d'initialiser l'audio"))
    }
}