//! Loom Media - Image, video, and audio support
//!
//! Phase L14: Media Playback
//!
//! ## Features
//! - **Images**: PNG, JPEG, WebP decoding with LRU cache
//! - **Video**: AV1 (rav1d), VP8/VP9 decoding, progressive download
//! - **Audio**: MP3, AAC, FLAC, WAV, Vorbis via symphonia
//! - **Controls**: Native UI for play/pause/volume/seek
//!
//! ## Example
//! ```rust
//! use loom_media::media_player::{MediaPlayer, MediaSource};
//!
//! // Create video player
//! let mut player = MediaPlayer::video_player();
//! 
//! // Load source
//! player.load(MediaSource::new("http://example.com/video.mp4"));
//! 
//! // Start playback
//! player.play();
//! 
//! // Get frame for rendering
//! if let Some(frame) = player.get_current_frame() {
//!     // Render frame.data (RGBA) to screen
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod audio;
pub mod hw_accel;
pub mod image;
pub mod media_player;
pub mod video;

// Re-export main types
pub use audio::*;
pub use image::*;
pub use media_player::*;
pub use video::*;

/// Version of the media crate
pub const VERSION: &str = "0.2.5-L16.5";

/// Check if video support is available
pub fn is_video_supported() -> bool {
    cfg!(feature = "video")
}

/// Check if audio support is available
pub fn is_audio_supported() -> bool {
    cfg!(feature = "audio")
}

/// Get supported video codecs
pub fn supported_video_codecs() -> &'static [video::VideoCodec] {
    &[
        video::VideoCodec::Av1,
        video::VideoCodec::Vp8,
        video::VideoCodec::Vp9,
    ]
}

/// Get supported audio codecs
pub fn supported_audio_codecs() -> &'static [audio::AudioCodec] {
    &[
        audio::AudioCodec::Mp3,
        audio::AudioCodec::Aac,
        audio::AudioCodec::Flac,
        audio::AudioCodec::Wav,
        audio::AudioCodec::Vorbis,
    ]
}

/// Detect media type from file extension
pub fn detect_media_type(path: &str) -> MediaType {
    if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos + 1..];
        
        // Video extensions
        if matches!(ext.to_lowercase().as_str(), "mp4" | "webm" | "avi" | "mkv" | "mov" | "av1") {
            return MediaType::Video;
        }
        
        // Audio extensions
        if matches!(ext.to_lowercase().as_str(), "mp3" | "aac" | "flac" | "wav" | "ogg" | "opus" | "m4a") {
            return MediaType::Audio;
        }
        
        // Image extensions
        if matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp") {
            return MediaType::Image;
        }
    }
    
    MediaType::Unknown
}

/// Media type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Unknown,
}

/// Initialize the media system
pub fn init() {
    // Initialize media codecs, threading, etc.
    // This is a no-op for now but provides a hook for future initialization
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.2.0-L14");
    }

    #[test]
    fn test_media_type_detection() {
        assert_eq!(detect_media_type("video.mp4"), MediaType::Video);
        assert_eq!(detect_media_type("audio.mp3"), MediaType::Audio);
        assert_eq!(detect_media_type("image.png"), MediaType::Image);
        assert_eq!(detect_media_type("unknown.xyz"), MediaType::Unknown);
    }

    #[test]
    fn test_codec_support() {
        let video_codecs = supported_video_codecs();
        assert!(!video_codecs.is_empty());
        
        let audio_codecs = supported_audio_codecs();
        assert!(!audio_codecs.is_empty());
    }
}
