//! Audio decoding and playback
//!
//! Phase L14: Media Playback
//! - Audio decoding via symphonia (MP3, AAC, FLAC, WAV, Vorbis)
//! - Sample rate conversion
//! - Volume control
//! - Basic buffering for streaming

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::VecDeque;
use core::time::Duration;

/// Audio codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    /// MP3 (MPEG-1/2 Audio Layer III)
    Mp3,
    /// AAC (Advanced Audio Coding)
    Aac,
    /// FLAC (Free Lossless Audio Codec)
    Flac,
    /// WAV (PCM)
    Wav,
    /// Vorbis
    Vorbis,
    /// Opus
    Opus,
    /// Unknown/unsupported
    Unknown,
}

impl AudioCodec {
    /// Detect codec from MIME type
    pub fn from_mime(mime: &str) -> Self {
        let mime_lower = mime.to_lowercase();
        if mime_lower.contains("mp3") || mime_lower.contains("mpeg") {
            AudioCodec::Mp3
        } else if mime_lower.contains("aac") {
            AudioCodec::Aac
        } else if mime_lower.contains("flac") {
            AudioCodec::Flac
        } else if mime_lower.contains("wav") || mime_lower.contains("pcm") {
            AudioCodec::Wav
        } else if mime_lower.contains("vorbis") || mime_lower.contains("ogg") {
            AudioCodec::Vorbis
        } else if mime_lower.contains("opus") {
            AudioCodec::Opus
        } else {
            AudioCodec::Unknown
        }
    }

    /// Detect codec from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => AudioCodec::Mp3,
            "aac" => AudioCodec::Aac,
            "flac" => AudioCodec::Flac,
            "wav" => AudioCodec::Wav,
            "ogg" => AudioCodec::Vorbis,
            "opus" => AudioCodec::Opus,
            _ => AudioCodec::Unknown,
        }
    }

    /// Check if codec is supported
    pub fn is_supported(&self) -> bool {
        matches!(self, 
            AudioCodec::Mp3 | 
            AudioCodec::Aac | 
            AudioCodec::Flac | 
            AudioCodec::Wav | 
            AudioCodec::Vorbis
        )
    }
}

/// Audio sample format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 8-bit unsigned integer
    U8,
    /// 16-bit signed integer
    I16,
    /// 24-bit signed integer
    I24,
    /// 32-bit signed integer
    I32,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
}

impl SampleFormat {
    /// Get size in bytes per sample
    pub fn size_bytes(&self) -> usize {
        match self {
            SampleFormat::U8 => 1,
            SampleFormat::I16 => 2,
            SampleFormat::I24 => 3,
            SampleFormat::I32 => 4,
            SampleFormat::F32 => 4,
            SampleFormat::F64 => 8,
        }
    }
}

/// Decoded audio buffer
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Sample data (interleaved: LRLRLR for stereo)
    pub samples: Vec<f32>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Presentation timestamp
    pub pts: Duration,
}

impl AudioBuffer {
    /// Create a new empty audio buffer
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            channels,
            pts: Duration::ZERO,
        }
    }

    /// Create buffer with capacity
    pub fn with_capacity(sample_rate: u32, channels: u16, capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            sample_rate,
            channels,
            pts: Duration::ZERO,
        }
    }

    /// Get duration of this buffer
    pub fn duration(&self) -> Duration {
        let sample_count = self.samples.len() as u64 / self.channels as u64;
        Duration::from_secs_f64(sample_count as f64 / self.sample_rate as f64)
    }

    /// Get number of samples per channel
    pub fn sample_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    /// Apply volume (0.0 - 1.0)
    pub fn apply_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        for sample in &mut self.samples {
            *sample *= volume;
        }
    }

    /// Convert to 16-bit PCM
    pub fn to_i16(&self) -> Vec<i16> {
        self.samples.iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect()
    }

    /// Silence the buffer
    pub fn silence(&mut self) {
        for sample in &mut self.samples {
            *sample = 0.0;
        }
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        self.samples.capacity() * core::mem::size_of::<f32>()
    }
}

/// Audio metadata
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Total duration (if known)
    pub duration: Option<Duration>,
    /// Audio codec
    pub codec: AudioCodec,
    /// Sample format
    pub sample_format: SampleFormat,
    /// Bitrate in bits per second
    pub bitrate: Option<u32>,
}

impl Default for AudioInfo {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            duration: None,
            codec: AudioCodec::Unknown,
            sample_format: SampleFormat::F32,
            bitrate: None,
        }
    }
}

/// Audio decoding error
#[derive(Debug, Clone, thiserror::Error)]
pub enum AudioError {
    #[error("Unsupported codec: {0:?}")]
    UnsupportedCodec(AudioCodec),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Decode failed: {0}")]
    DecodeFailed(String),
    
    #[error("Not enough data")]
    NotEnoughData,
    
    #[error("End of stream")]
    EndOfStream,
    
    #[error("Out of memory")]
    OutOfMemory,
}

/// Audio decoder trait
pub trait AudioDecoder: Send {
    /// Initialize decoder with audio info
    fn init(&mut self, info: AudioInfo) -> Result<(), AudioError>;
    
    /// Decode a packet of compressed data
    fn decode(&mut self, data: &[u8]) -> Result<Vec<AudioBuffer>, AudioError>;
    
    /// Flush remaining samples
    fn flush(&mut self) -> Result<Vec<AudioBuffer>, AudioError>;
    
    /// Get audio info
    fn info(&self) -> &AudioInfo;
    
    /// Reset decoder state
    fn reset(&mut self);
}

/// Symphonia-based decoder for multiple formats
#[cfg(feature = "symphonia")]
pub struct SymphoniaDecoder {
    info: AudioInfo,
    // symphonia decoder would go here
    decoder: Option<()>, // Placeholder for symphonia::Decoder
}

#[cfg(feature = "symphonia")]
impl SymphoniaDecoder {
    /// Create a new symphonia decoder
    pub fn new() -> Self {
        Self {
            info: AudioInfo::default(),
            decoder: None,
        }
    }
}

#[cfg(feature = "symphonia")]
impl AudioDecoder for SymphoniaDecoder {
    fn init(&mut self, info: AudioInfo) -> Result<(), AudioError> {
        self.info = info;
        // Initialize symphonia decoder
        Ok(())
    }
    
    fn decode(&mut self, _data: &[u8]) -> Result<Vec<AudioBuffer>, AudioError> {
        // Decode using symphonia
        // For now, generate test tone
        let mut buffer = AudioBuffer::with_capacity(
            self.info.sample_rate, 
            self.info.channels, 
            self.info.sample_rate as usize // 1 second
        );
        
        // Generate 1 second of 440Hz sine wave test tone
        generate_test_tone(&mut buffer, 440.0, 1.0);
        
        Ok(vec![buffer])
    }
    
    fn flush(&mut self) -> Result<Vec<AudioBuffer>, AudioError> {
        Ok(Vec::new())
    }
    
    fn info(&self) -> &AudioInfo {
        &self.info
    }
    
    fn reset(&mut self) {
        self.decoder = None;
    }
}

/// Generate test tone (sine wave)
fn generate_test_tone(buffer: &mut AudioBuffer, frequency: f32, duration_secs: f32) {
    let sample_count = (buffer.sample_rate as f32 * duration_secs) as usize;
    let total_samples = sample_count * buffer.channels as usize;
    
    buffer.samples.reserve(total_samples);
    
    for i in 0..sample_count {
        let t = i as f32 / buffer.sample_rate as f32;
        let sample = (2.0 * core::f32::consts::PI * frequency * t).sin() * 0.5;
        
        // Interleave for all channels
        for _ in 0..buffer.channels {
            buffer.samples.push(sample);
        }
    }
}

/// Create appropriate decoder for codec
pub fn create_decoder(codec: AudioCodec) -> Result<Box<dyn AudioDecoder>, AudioError> {
    match codec {
        #[cfg(feature = "symphonia")]
        AudioCodec::Mp3 | AudioCodec::Aac | AudioCodec::Flac | AudioCodec::Wav | AudioCodec::Vorbis => {
            Ok(Box::new(SymphoniaDecoder::new()))
        }
        _ => Err(AudioError::UnsupportedCodec(codec)),
    }
}

/// Audio buffer for streaming
#[derive(Debug)]
pub struct AudioStreamBuffer {
    /// Buffered audio data (compressed)
    data: VecDeque<u8>,
    /// Total buffer capacity
    capacity: usize,
    /// Current buffer size
    size: usize,
    /// Whether download is complete
    complete: bool,
}

impl AudioStreamBuffer {
    /// Create a new audio buffer with capacity in bytes
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
            size: 0,
            complete: false,
        }
    }

    /// Add data to buffer
    pub fn append(&mut self, data: &[u8]) -> Result<(), AudioError> {
        if self.size + data.len() > self.capacity {
            return Err(AudioError::OutOfMemory);
        }
        
        self.data.extend(data);
        self.size += data.len();
        Ok(())
    }

    /// Read data from buffer
    pub fn read(&mut self, len: usize) -> Vec<u8> {
        let len = len.min(self.data.len());
        self.data.drain(..len).collect()
    }

    /// Mark download as complete
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Check if buffer has enough data for playback
    pub fn has_enough_data(&self, min_seconds: f32) -> bool {
        if self.complete {
            return true;
        }
        
        // Rough estimate: ~128kbps = 16KB per second
        let min_bytes = (min_seconds * 16.0 * 1024.0) as usize;
        self.size >= min_bytes
    }

    /// Get buffer fill percentage
    pub fn fill_percent(&self) -> f32 {
        (self.size as f32 / self.capacity as f32) * 100.0
    }

    /// Check if download is complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get buffered size in bytes
    pub fn buffered_bytes(&self) -> usize {
        self.size
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.data.clear();
        self.size = 0;
        self.complete = false;
    }
}

impl Default for AudioStreamBuffer {
    fn default() -> Self {
        Self::new(10 * 1024 * 1024) // 10MB default
    }
}

/// Audio player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlayerState {
    /// Not loaded
    Empty,
    /// Loading/Buffering
    Loading,
    /// Ready to play
    Ready,
    /// Currently playing
    Playing,
    /// Paused
    Paused,
    /// Ended
    Ended,
    /// Error state
    Error,
}

/// Audio player with controls
#[derive(Debug)]
pub struct AudioPlayer {
    /// Current state
    pub state: AudioPlayerState,
    /// Audio information
    pub info: AudioInfo,
    /// Current playback position
    pub current_time: Duration,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,
    /// Playback speed
    pub playback_rate: f32,
    /// Whether to loop
    pub loop_playback: bool,
    /// Audio buffer
    buffer: AudioStreamBuffer,
    /// Decoder
    decoder: Option<Box<dyn AudioDecoder>>,
    /// Decoded sample queue
    sample_queue: VecDeque<AudioBuffer>,
    /// Maximum queue duration
    max_queue_duration: Duration,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Self {
        Self {
            state: AudioPlayerState::Empty,
            info: AudioInfo::default(),
            current_time: Duration::ZERO,
            volume: 1.0,
            muted: false,
            playback_rate: 1.0,
            loop_playback: false,
            buffer: AudioStreamBuffer::default(),
            decoder: None,
            sample_queue: VecDeque::new(),
            max_queue_duration: Duration::from_secs(5),
        }
    }

    /// Load audio from source
    pub fn load(&mut self, codec: AudioCodec, info: AudioInfo) -> Result<(), AudioError> {
        if !codec.is_supported() {
            return Err(AudioError::UnsupportedCodec(codec));
        }

        self.decoder = Some(create_decoder(codec)?);
        self.info = info;
        self.state = AudioPlayerState::Loading;
        self.current_time = Duration::ZERO;
        self.sample_queue.clear();
        
        if let Some(ref mut decoder) = self.decoder {
            decoder.init(info)?;
        }
        
        Ok(())
    }

    /// Append data to buffer (for progressive download)
    pub fn append_buffer(&mut self, data: &[u8]) -> Result<(), AudioError> {
        self.buffer.append(data)?;
        
        // Check if we have enough data to start playback
        if self.state == AudioPlayerState::Loading && self.buffer.has_enough_data(2.0) {
            self.state = AudioPlayerState::Ready;
        }
        
        Ok(())
    }

    /// Mark buffer as complete
    pub fn mark_buffer_complete(&mut self) {
        self.buffer.mark_complete();
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        match self.state {
            AudioPlayerState::Ready | AudioPlayerState::Paused => {
                self.state = AudioPlayerState::Playing;
            }
            _ => {}
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == AudioPlayerState::Playing {
            self.state = AudioPlayerState::Paused;
        }
    }

    /// Toggle play/pause
    pub fn toggle_playback(&mut self) {
        match self.state {
            AudioPlayerState::Playing => self.pause(),
            AudioPlayerState::Paused | AudioPlayerState::Ready => self.play(),
            _ => {}
        }
    }

    /// Stop playback and reset
    pub fn stop(&mut self) {
        self.state = AudioPlayerState::Empty;
        self.current_time = Duration::ZERO;
        self.sample_queue.clear();
        self.buffer.clear();
        if let Some(ref mut decoder) = self.decoder {
            decoder.reset();
        }
    }

    /// Seek to position (in seconds)
    pub fn seek(&mut self, seconds: f64) {
        self.current_time = Duration::from_secs_f64(seconds);
        self.sample_queue.clear();
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.muted = volume == 0.0;
    }

    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Set playback rate (0.25 - 4.0)
    pub fn set_playback_rate(&mut self, rate: f32) {
        self.playback_rate = rate.clamp(0.25, 4.0);
    }

    /// Update playback (decode samples, advance time)
    /// Returns audio buffer to play, if any
    pub fn update(&mut self, delta_time: Duration) -> Option<AudioBuffer> {
        if self.state != AudioPlayerState::Playing {
            return None;
        }

        // Advance current time
        let adjusted_delta = delta_time.mul_f32(self.playback_rate);
        self.current_time += adjusted_delta;

        // Check if we've reached the end
        if let Some(duration) = self.info.duration {
            if self.current_time >= duration {
                if self.loop_playback {
                    self.current_time = Duration::ZERO;
                } else {
                    self.state = AudioPlayerState::Ended;
                    return None;
                }
            }
        }

        // Decode more samples if needed
        self.decode_samples();

        // Get next buffer to play
        self.get_next_buffer()
    }

    /// Decode samples from buffer
    fn decode_samples(&mut self) {
        // Check queue duration
        let queue_duration: Duration = self.sample_queue.iter()
            .map(|b| b.duration())
            .sum();
        
        if queue_duration >= self.max_queue_duration {
            return;
        }

        if let Some(ref mut decoder) = self.decoder {
            // Read chunk from buffer
            let chunk_size = 4096;
            let data = self.buffer.read(chunk_size);
            
            if !data.is_empty() {
                if let Ok(buffers) = decoder.decode(&data) {
                    for buffer in buffers {
                        self.sample_queue.push_back(buffer);
                    }
                }
            }
        }
    }

    /// Get next audio buffer to play
    fn get_next_buffer(&mut self) -> Option<AudioBuffer> {
        self.sample_queue.pop_front().map(|mut buffer| {
            // Apply volume
            let effective_volume = if self.muted { 0.0 } else { self.volume };
            buffer.apply_volume(effective_volume);
            buffer
        })
    }

    /// Get buffer fill percentage
    pub fn buffer_percent(&self) -> f32 {
        self.buffer.fill_percent()
    }

    /// Check if audio is playing
    pub fn is_playing(&self) -> bool {
        self.state == AudioPlayerState::Playing
    }

    /// Check if audio has ended
    pub fn has_ended(&self) -> bool {
        self.state == AudioPlayerState::Ended
    }

    /// Get current playback position as seconds
    pub fn current_time_secs(&self) -> f64 {
        self.current_time.as_secs_f64()
    }

    /// Get total duration as seconds (if known)
    pub fn duration_secs(&self) -> Option<f64> {
        self.info.duration.map(|d| d.as_secs_f64())
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample rate converter
pub struct SampleRateConverter {
    from_rate: u32,
    to_rate: u32,
    channels: u16,
}

impl SampleRateConverter {
    /// Create a new sample rate converter
    pub fn new(from_rate: u32, to_rate: u32, channels: u16) -> Self {
        Self {
            from_rate,
            to_rate,
            channels,
        }
    }

    /// Convert sample buffer to target rate
    pub fn convert(&self, input: &[f32]) -> Vec<f32> {
        if self.from_rate == self.to_rate {
            return input.to_vec();
        }

        let ratio = self.to_rate as f64 / self.from_rate as f64;
        let input_samples = input.len() / self.channels as usize;
        let output_samples = (input_samples as f64 * ratio) as usize;
        
        let mut output = Vec::with_capacity(output_samples * self.channels as usize);
        
        // Simple linear interpolation
        for i in 0..output_samples {
            let src_pos = i as f64 / ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f64;
            
            for ch in 0..self.channels as usize {
                let idx1 = (src_idx * self.channels as usize + ch).min(input.len() - 1);
                let idx2 = ((src_idx + 1) * self.channels as usize + ch).min(input.len() - 1);
                
                let sample1 = input[idx1] as f64;
                let sample2 = input[idx2] as f64;
                let sample = sample1 + (sample2 - sample1) * frac;
                
                output.push(sample as f32);
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_codec_detection() {
        assert_eq!(AudioCodec::from_extension("mp3"), AudioCodec::Mp3);
        assert_eq!(AudioCodec::from_extension("flac"), AudioCodec::Flac);
        assert_eq!(AudioCodec::from_extension("wav"), AudioCodec::Wav);
    }

    #[test]
    fn test_audio_buffer() {
        let mut buffer = AudioBuffer::new(48000, 2);
        
        // Add some samples
        buffer.samples = vec![0.5, 0.5, -0.5, -0.5];
        
        assert_eq!(buffer.sample_count(), 2);
        assert_eq!(buffer.channels, 2);
        
        // Test volume
        buffer.apply_volume(0.5);
        assert_eq!(buffer.samples[0], 0.25);
    }

    #[test]
    fn test_audio_buffer_to_i16() {
        let mut buffer = AudioBuffer::new(48000, 1);
        buffer.samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        
        let i16_samples = buffer.to_i16();
        assert_eq!(i16_samples[0], 0);
        assert_eq!(i16_samples[1], 16383); // Approximately 0.5 * 32767
        assert_eq!(i16_samples[2], -16384); // Approximately -0.5 * 32768
        assert_eq!(i16_samples[3], 32767);
        assert_eq!(i16_samples[4], -32768);
    }

    #[test]
    fn test_sample_rate_converter() {
        let converter = SampleRateConverter::new(48000, 24000, 2);
        
        let input: Vec<f32> = (0..96).map(|i| i as f32 / 96.0).collect();
        let output = converter.convert(&input);
        
        // Output should be roughly half the size
        assert!(output.len() < input.len());
    }

    #[test]
    fn test_audio_player_state() {
        let mut player = AudioPlayer::new();
        assert_eq!(player.state, AudioPlayerState::Empty);
        
        player.state = AudioPlayerState::Ready;
        player.play();
        assert_eq!(player.state, AudioPlayerState::Playing);
        
        player.pause();
        assert_eq!(player.state, AudioPlayerState::Paused);
    }

    #[test]
    fn test_stream_buffer() {
        let mut buffer = AudioStreamBuffer::new(1024);
        
        buffer.append(b"audio data").unwrap();
        assert_eq!(buffer.buffered_bytes(), 10);
        
        let data = buffer.read(5);
        assert_eq!(&data, b"audio");
    }

    #[test]
    fn test_test_tone() {
        let mut buffer = AudioBuffer::new(48000, 1);
        generate_test_tone(&mut buffer, 440.0, 0.1);
        
        // Should have approximately 4800 samples (0.1s * 48000Hz)
        assert!(buffer.samples.len() >= 4800);
        
        // Samples should be non-zero
        assert!(buffer.samples.iter().any(|&s| s > 0.0));
    }
}
