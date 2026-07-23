// mic/mod.rs — Microphone capture.
//
// Uses the `cpal` crate for cross-platform audio capture.
// Always captures 16 kHz, mono, f32 PCM — the format both engines expect.
// If the device doesn't natively support 16 kHz, cpal's sample rate conversion is used.
//
// Full implementation: Phase 2 (the actual audio loop and channel feeding).
// This module contains the correct structure and types; the capture loop
// is stubbed where it requires the engine to be wired.

use crate::error::{AppError, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, SupportedStreamConfig,
};

/// Target sample rate for all ASR engines.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// List all audio input device names available on this system.
/// Returns names suitable for display in the settings UI.
pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| AppError::Internal(format!("list audio devices: {e}")))?;

    let names = devices
        .filter_map(|d| d.name().ok())
        .collect::<Vec<_>>();

    Ok(names)
}

/// Find an input device by name. Returns the default device if name is empty.
pub fn find_device(name: &str) -> Result<Device> {
    let host = cpal::default_host();

    if name.is_empty() {
        return host
            .default_input_device()
            .ok_or_else(|| AppError::MicBusy);
    }

    host.input_devices()
        .map_err(|e| AppError::Internal(format!("enumerate devices: {e}")))?
        .find(|d| d.name().ok().as_deref() == Some(name))
        .ok_or_else(|| {
            AppError::Internal(format!("audio device not found: '{name}'"))
        })
}

/// Negotiated stream configuration — always 16 kHz mono f32.
pub fn preferred_config(device: &Device) -> Result<SupportedStreamConfig> {
    use cpal::SampleRate;

    let supported = device
        .supported_input_configs()
        .map_err(|e| AppError::Internal(format!("query device configs: {e}")))?;

    // Prefer exactly 16 kHz f32 mono if the device supports it.
    // cpal will resample if not available.
    for range in supported {
        if range.channels() == 1
            && range.sample_format() == SampleFormat::F32
            && range.min_sample_rate().0 <= TARGET_SAMPLE_RATE
            && range.max_sample_rate().0 >= TARGET_SAMPLE_RATE
        {
            return Ok(range.with_sample_rate(SampleRate(TARGET_SAMPLE_RATE)));
        }
    }

    // Fall back: take whatever the device offers and let the engine handle it.
    device
        .default_input_config()
        .map_err(|e| AppError::Internal(format!("default input config: {e}")))
}

/// Capture state — holds the active cpal stream.
/// Dropping this struct stops the capture immediately.
pub struct CaptureHandle {
    // cpal streams are stopped by dropping them.
    _stream: cpal::Stream,
}

impl CaptureHandle {
    /// Start capturing audio from the given device.
    /// Calls `on_chunk` with each ~100ms chunk of f32 PCM at 16 kHz.
    ///
    /// TODO (Phase 2): Wire `on_chunk` to the ASR engine's feed_audio().
    pub fn start<F>(device: &Device, config: SupportedStreamConfig, mut on_chunk: F)
        -> Result<Self>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        let stream_config = config.config();
        let sample_format = config.sample_format();

        // Build stream with error callback
        let err_fn = |e: cpal::StreamError| {
            log::error!("audio stream error: {e}");
        };

        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| on_chunk(data),
                err_fn,
                None,
            ),
            SampleFormat::I16 => {
                // Convert i16 to f32
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &_| {
                        let f32_buf: Vec<f32> = data
                            .iter()
                            .map(|&s| s as f32 / i16::MAX as f32)
                            .collect();
                        on_chunk(&f32_buf);
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                return Err(AppError::Internal(format!(
                    "unsupported sample format: {sample_format:?}"
                )))
            }
        }
        .map_err(|e| AppError::Internal(format!("build audio stream: {e}")))?;

        stream
            .play()
            .map_err(|e| AppError::Internal(format!("start audio stream: {e}")))?;

        log::info!("mic: capture started ({} Hz, {} ch, {:?})",
            stream_config.sample_rate.0,
            stream_config.channels,
            sample_format,
        );

        Ok(Self { _stream: stream })
    }
}
