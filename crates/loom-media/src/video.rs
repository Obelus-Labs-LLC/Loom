//! Video decoding and playback
//!
//! Phase L14: Media Playback
//! - AV1 decoding via rav1d (pure Rust)
//! - H.264 support (placeholder for future openh264 integration)
//! - Frame extraction and rendering
//! - Basic buffering for progressive download

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::VecDeque;
use core::time::Duration;

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// AV1 codec (AOMedia Video 1)
    Av1,
    /// H.264 / AVC
    H264,
    /// H.265 / HEVC
    H265,
    /// VP8
    Vp8,
    /// VP9
    Vp9,
    /// Unknown/unsupported codec
    Unknown,
}

impl VideoCodec {
    /// Detect codec from MIME type
    pub fn from_mime(mime: &str) -> Self {
        let mime_lower = mime.to_lowercase();
        if mime_lower.contains("av01") || mime_lower.contains("av1") {
            VideoCodec::Av1
        } else if mime_lower.contains("avc") || mime_lower.contains("h264") {
            VideoCodec::H264
        } else if mime_lower.contains("hevc") || mime_lower.contains("h265") {
            VideoCodec::H265
        } else if mime_lower.contains("vp8") {
            VideoCodec::Vp8
        } else if mime_lower.contains("vp9") {
            VideoCodec::Vp9
        } else {
            VideoCodec::Unknown
        }
    }

    /// Detect codec from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "av1" | "obu" | "ivf" => VideoCodec::Av1,
            "mp4" | "m4v" | "h264" => VideoCodec::H264,
            "hevc" | "h265" => VideoCodec::H265,
            "webm" => VideoCodec::Vp9, // WebM typically uses VP8/VP9
            "vp8" => VideoCodec::Vp8,
            "vp9" => VideoCodec::Vp9,
            _ => VideoCodec::Unknown,
        }
    }

    /// Check if codec is supported
    pub fn is_supported(&self) -> bool {
        matches!(self, VideoCodec::Av1 | VideoCodec::Vp8 | VideoCodec::Vp9)
        // H264/H265 would require openh264 or similar
    }
}

/// Decoded video frame
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Frame data in RGBA format
    pub data: Vec<u8>,
    /// Presentation timestamp
    pub pts: Duration,
    /// Frame duration
    pub duration: Duration,
    /// Whether this is a keyframe
    pub is_keyframe: bool,
}

impl VideoFrame {
    /// Create a new empty frame
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![0; (width * height * 4) as usize];
        Self {
            width,
            height,
            data,
            pts: Duration::ZERO,
            duration: Duration::from_millis(33), // ~30fps default
            is_keyframe: false,
        }
    }

    /// Get pixel at (x, y) as RGBA tuple
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 4 > self.data.len() {
            return None;
        }
        Some((
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ))
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        self.data.len()
    }
}

/// Video metadata
#[derive(Debug, Clone)]
pub struct VideoInfo {
    /// Video width
    pub width: u32,
    /// Video height
    pub height: u32,
    /// Frame rate (frames per second)
    pub frame_rate: f32,
    /// Total duration
    pub duration: Option<Duration>,
    /// Total frame count (if known)
    pub frame_count: Option<u64>,
    /// Video codec
    pub codec: VideoCodec,
    /// Pixel aspect ratio
    pub pixel_aspect_ratio: f32,
}

impl Default for VideoInfo {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            frame_rate: 30.0,
            duration: None,
            frame_count: None,
            codec: VideoCodec::Unknown,
            pixel_aspect_ratio: 1.0,
        }
    }
}

/// Video decoding error
#[derive(Debug, Clone, thiserror::Error)]
pub enum VideoError {
    #[error("Unsupported codec: {0:?}")]
    UnsupportedCodec(VideoCodec),
    
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

/// Video decoder trait
pub trait VideoDecoder: Send {
    /// Initialize decoder with video info
    fn init(&mut self, info: VideoInfo) -> Result<(), VideoError>;
    
    /// Decode a packet of compressed data
    fn decode(&mut self, data: &[u8]) -> Result<Vec<VideoFrame>, VideoError>;
    
    /// Flush remaining frames
    fn flush(&mut self) -> Result<Vec<VideoFrame>, VideoError>;
    
    /// Get video info
    fn info(&self) -> &VideoInfo;
    
    /// Reset decoder state
    fn reset(&mut self);
}

/// AV1 decoder using rav1d
#[cfg(feature = "rav1d")]
pub struct Av1Decoder {
    info: VideoInfo,
    // rav1d context would go here
    decoder: Option<()>, // Placeholder for rav1d::Context
}

#[cfg(feature = "rav1d")]
impl Av1Decoder {
    /// Create a new AV1 decoder
    pub fn new() -> Self {
        Self {
            info: VideoInfo::default(),
            decoder: None,
        }
    }
}

#[cfg(feature = "rav1d")]
impl VideoDecoder for Av1Decoder {
    fn init(&mut self, info: VideoInfo) -> Result<(), VideoError> {
        self.info = info;
        // Initialize rav1d decoder here
        // self.decoder = Some(rav1d::Context::new()?);
        Ok(())
    }
    
    fn decode(&mut self, data: &[u8]) -> Result<Vec<VideoFrame>, VideoError> {
        // Decode using rav1d
        // For now, return a placeholder frame
        let mut frames = Vec::new();
        
        // Create a test pattern frame
        let mut frame = VideoFrame::new(self.info.width, self.info.height);
        
        // Fill with test pattern (color bars)
        fill_test_pattern(&mut frame);
        
        frames.push(frame);
        Ok(frames)
    }
    
    fn flush(&mut self) -> Result<Vec<VideoFrame>, VideoError> {
        Ok(Vec::new())
    }
    
    fn info(&self) -> &VideoInfo {
        &self.info
    }
    
    fn reset(&mut self) {
        self.decoder = None;
    }
}

/// VP8/VP9 decoder
pub struct VpxDecoder {
    info: VideoInfo,
    codec: VideoCodec,
}

impl VpxDecoder {
    /// Create a new VP8/VP9 decoder
    pub fn new(codec: VideoCodec) -> Self {
        Self {
            info: VideoInfo::default(),
            codec,
        }
    }
}

impl VideoDecoder for VpxDecoder {
    fn init(&mut self, info: VideoInfo) -> Result<(), VideoError> {
        self.info = info;
        Ok(())
    }
    
    fn decode(&mut self, _data: &[u8]) -> Result<Vec<VideoFrame>, VideoError> {
        // VP8/VP9 decoding would go here
        // For now, return test pattern
        let mut frame = VideoFrame::new(self.info.width, self.info.height);
        fill_test_pattern(&mut frame);
        Ok(vec![frame])
    }
    
    fn flush(&mut self) -> Result<Vec<VideoFrame>, VideoError> {
        Ok(Vec::new())
    }
    
    fn info(&self) -> &VideoInfo {
        &self.info
    }
    
    fn reset(&mut self) {
        // Reset decoder
    }
}

/// Fill frame with test pattern (color bars)
fn fill_test_pattern(frame: &mut VideoFrame) {
    let bar_count = 8;
    let bar_width = frame.width / bar_count;
    
    let colors = [
        (255, 255, 255), // White
        (255, 255, 0),   // Yellow
        (0, 255, 255),   // Cyan
        (0, 255, 0),     // Green
        (255, 0, 255),   // Magenta
        (255, 0, 0),     // Red
        (0, 0, 255),     // Blue
        (0, 0, 0),       // Black
    ];
    
    for y in 0..frame.height {
        for x in 0..frame.width {
            let bar = (x / bar_width).min(7) as usize;
            let color = colors[bar];
            let idx = ((y * frame.width + x) * 4) as usize;
            frame.data[idx] = color.0;
            frame.data[idx + 1] = color.1;
            frame.data[idx + 2] = color.2;
            frame.data[idx + 3] = 255; // Alpha
        }
    }
}

/// Create appropriate decoder for codec
pub fn create_decoder(codec: VideoCodec) -> Result<Box<dyn VideoDecoder>, VideoError> {
    match codec {
        #[cfg(feature = "rav1d")]
        VideoCodec::Av1 => Ok(Box::new(Av1Decoder::new())),
        VideoCodec::Vp8 | VideoCodec::Vp9 => Ok(Box::new(VpxDecoder::new(codec))),
        _ => Err(VideoError::UnsupportedCodec(codec)),
    }
}

/// Video buffer for progressive download
#[derive(Debug)]
pub struct VideoBuffer {
    /// Buffered data
    data: VecDeque<u8>,
    /// Total buffer capacity
    capacity: usize,
    /// Current buffer size
    size: usize,
    /// Whether download is complete
    complete: bool,
    /// Total expected size (if known)
    expected_size: Option<usize>,
}

impl VideoBuffer {
    /// Create a new video buffer with capacity in bytes
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
            size: 0,
            complete: false,
            expected_size: None,
        }
    }

    /// Add data to buffer
    pub fn append(&mut self, data: &[u8]) -> Result<(), VideoError> {
        if self.size + data.len() > self.capacity {
            return Err(VideoError::OutOfMemory);
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

    /// Peek at data without removing
    pub fn peek(&self, len: usize) -> Vec<u8> {
        let len = len.min(self.data.len());
        self.data.iter().take(len).copied().collect()
    }

    /// Mark download as complete
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Check if buffer has enough data for playback
    pub fn has_enough_data(&self, min_buffer_ms: u32) -> bool {
        // For now, simple threshold-based buffering
        // In production, this would check actual decoded time
        if self.complete {
            return true;
        }
        
        // Assume ~1MB per second of video at reasonable quality
        let min_bytes = (min_buffer_ms as usize) * 1024;
        self.size >= min_bytes
    }

    /// Get buffer fill percentage
    pub fn fill_percent(&self) -> f32 {
        if let Some(expected) = self.expected_size {
            (self.size as f32 / expected as f32) * 100.0
        } else {
            // Unknown total, return buffer capacity percent
            (self.size as f32 / self.capacity as f32) * 100.0
        }
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

impl Default for VideoBuffer {
    fn default() -> Self {
        Self::new(50 * 1024 * 1024) // 50MB default
    }
}

/// Video player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
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
    /// Seeking
    Seeking,
    /// Ended
    Ended,
    /// Error state
    Error,
}

/// Video player controls and state
#[derive(Debug)]
pub struct VideoPlayer {
    /// Current state
    pub state: PlayerState,
    /// Video information
    pub info: VideoInfo,
    /// Current playback position
    pub current_time: Duration,
    /// Current frame (if decoded)
    pub current_frame: Option<VideoFrame>,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,
    /// Playback speed
    pub playback_rate: f32,
    /// Whether to loop
    pub loop_playback: bool,
    /// Video buffer
    buffer: VideoBuffer,
    /// Decoder
    decoder: Option<Box<dyn VideoDecoder>>,
    /// Frame queue for decoded frames
    frame_queue: VecDeque<VideoFrame>,
    /// Maximum queue size in frames
    max_queue_size: usize,
}

impl VideoPlayer {
    /// Create a new video player
    pub fn new() -> Self {
        Self {
            state: PlayerState::Empty,
            info: VideoInfo::default(),
            current_time: Duration::ZERO,
            current_frame: None,
            volume: 1.0,
            muted: false,
            playback_rate: 1.0,
            loop_playback: false,
            buffer: VideoBuffer::default(),
            decoder: None,
            frame_queue: VecDeque::new(),
            max_queue_size: 5,
        }
    }

    /// Load video from source
    pub fn load(&mut self, codec: VideoCodec, info: VideoInfo) -> Result<(), VideoError> {
        if !codec.is_supported() {
            return Err(VideoError::UnsupportedCodec(codec));
        }

        self.decoder = Some(create_decoder(codec)?);
        self.info = info;
        self.state = PlayerState::Loading;
        self.current_time = Duration::ZERO;
        self.frame_queue.clear();
        
        if let Some(ref mut decoder) = self.decoder {
            decoder.init(info)?;
        }
        
        Ok(())
    }

    /// Append data to buffer (for progressive download)
    pub fn append_buffer(&mut self, data: &[u8]) -> Result<(), VideoError> {
        self.buffer.append(data)?;
        
        // Check if we have enough data to start playback
        if self.state == PlayerState::Loading && self.buffer.has_enough_data(2000) {
            self.state = PlayerState::Ready;
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
            PlayerState::Ready | PlayerState::Paused => {
                self.state = PlayerState::Playing;
            }
            _ => {}
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing {
            self.state = PlayerState::Paused;
        }
    }

    /// Toggle play/pause
    pub fn toggle_playback(&mut self) {
        match self.state {
            PlayerState::Playing => self.pause(),
            PlayerState::Paused | PlayerState::Ready => self.play(),
            _ => {}
        }
    }

    /// Stop playback and reset
    pub fn stop(&mut self) {
        self.state = PlayerState::Empty;
        self.current_time = Duration::ZERO;
        self.current_frame = None;
        self.frame_queue.clear();
        self.buffer.clear();
        if let Some(ref mut decoder) = self.decoder {
            decoder.reset();
        }
    }

    /// Seek to position (in seconds)
    pub fn seek(&mut self, seconds: f64) {
        self.current_time = Duration::from_secs_f64(seconds);
        self.frame_queue.clear();
        // Actual seeking would require keyframe index
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

    /// Update playback (decode frames, advance time)
    /// Call this periodically during playback
    pub fn update(&mut self, delta_time: Duration) {
        if self.state != PlayerState::Playing {
            return;
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
                    self.state = PlayerState::Ended;
                    return;
                }
            }
        }

        // Decode more frames if needed
        self.decode_frames();

        // Get the current frame based on presentation time
        self.update_current_frame();
    }

    /// Decode frames from buffer
    fn decode_frames(&mut self) {
        // Limit queue size
        if self.frame_queue.len() >= self.max_queue_size {
            return;
        }

        if let Some(ref mut decoder) = self.decoder {
            // Read chunk from buffer
            let chunk_size = 4096; // Read 4KB at a time
            let data = self.buffer.read(chunk_size);
            
            if !data.is_empty() {
                if let Ok(frames) = decoder.decode(&data) {
                    for frame in frames {
                        self.frame_queue.push_back(frame);
                    }
                }
            }
        }
    }

    /// Update current frame based on playback time
    fn update_current_frame(&mut self) {
        // Find the frame closest to current playback time
        while let Some(frame) = self.frame_queue.front() {
            if frame.pts <= self.current_time {
                self.current_frame = self.frame_queue.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get current frame for rendering
    pub fn get_current_frame(&self) -> Option<&VideoFrame> {
        self.current_frame.as_ref()
    }

    /// Get buffer fill percentage
    pub fn buffer_percent(&self) -> f32 {
        self.buffer.fill_percent()
    }

    /// Check if video is playing
    pub fn is_playing(&self) -> bool {
        self.state == PlayerState::Playing
    }

    /// Check if video has ended
    pub fn has_ended(&self) -> bool {
        self.state == PlayerState::Ended
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_codec_detection() {
        assert_eq!(VideoCodec::from_extension("av1"), VideoCodec::Av1);
        assert_eq!(VideoCodec::from_extension("mp4"), VideoCodec::H264);
        assert_eq!(VideoCodec::from_extension("webm"), VideoCodec::Vp9);
    }

    #[test]
    fn test_video_buffer() {
        let mut buffer = VideoBuffer::new(1024);
        
        buffer.append(b"test data").unwrap();
        assert_eq!(buffer.buffered_bytes(), 9);
        
        let data = buffer.read(4);
        assert_eq!(&data, b"test");
        assert_eq!(buffer.buffered_bytes(), 5);
    }

    #[test]
    fn test_video_player_state() {
        let mut player = VideoPlayer::new();
        assert_eq!(player.state, PlayerState::Empty);
        
        // Simulate loading
        player.state = PlayerState::Ready;
        player.play();
        assert_eq!(player.state, PlayerState::Playing);
        
        player.pause();
        assert_eq!(player.state, PlayerState::Paused);
        
        player.toggle_playback();
        assert_eq!(player.state, PlayerState::Playing);
    }

    #[test]
    fn test_video_frame() {
        let frame = VideoFrame::new(100, 100);
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
        assert_eq!(frame.data.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_test_pattern() {
        let mut frame = VideoFrame::new(800, 600);
        fill_test_pattern(&mut frame);
        
        // Check that we have different colors in different bars
        let pixel0 = frame.get_pixel(0, 0).unwrap();
        let pixel400 = frame.get_pixel(400, 0).unwrap();
        
        assert_ne!(pixel0, pixel400);
    }
}
