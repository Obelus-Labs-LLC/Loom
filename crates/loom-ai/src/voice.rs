//! Voice Input - Speech-to-Text Integration Stub
//!
//! Phase L18: Prepares for L12.5 voice input integration

use alloc::string::{String, ToString};

/// Voice input state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    /// Voice input inactive
    Inactive,
    /// Listening for voice input
    Listening,
    /// Processing speech to text
    Processing,
    /// Voice input complete
    Complete,
    /// Error occurred
    Error,
}

/// Voice input handler
#[derive(Debug, Clone)]
pub struct VoiceInput {
    /// Current state
    state: VoiceState,
    /// Transcribed text
    transcript: String,
    /// Confidence score
    confidence: f32,
    /// Error message (if any)
    error: Option<String>,
    /// Language code (e.g., "en-US")
    language: String,
    /// Continuous listening mode
    continuous: bool,
}

impl VoiceInput {
    /// Create new voice input handler
    pub fn new() -> Self {
        Self {
            state: VoiceState::Inactive,
            transcript: String::new(),
            confidence: 0.0,
            error: None,
            language: "en-US".to_string(),
            continuous: false,
        }
    }

    /// Start listening for voice input
    pub fn start(&mut self) -> Result<(), VoiceError> {
        // STUB: In L12.5, this will activate microphone
        // and start streaming audio to STT service
        
        self.state = VoiceState::Listening;
        self.transcript.clear();
        self.confidence = 0.0;
        self.error = None;
        
        log::info!("Voice input started (stub)");
        Ok(())
    }

    /// Stop listening
    pub fn stop(&mut self) {
        if self.state == VoiceState::Listening {
            self.state = VoiceState::Processing;
            
            // STUB: In L12.5, this will finalize audio capture
            // and send to STT service for transcription
            
            log::info!("Voice input stopped, processing (stub)");
        }
    }

    /// Get current state
    pub fn state(&self) -> VoiceState {
        self.state
    }

    /// Check if currently listening
    pub fn is_listening(&self) -> bool {
        self.state == VoiceState::Listening
    }

    /// Get transcribed text
    pub fn transcript(&self) -> &str {
        &self.transcript
    }

    /// Get confidence score
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// Set language
    pub fn set_language(&mut self, lang: impl Into<String>) {
        self.language = lang.into();
    }

    /// Get language
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Set continuous mode
    pub fn set_continuous(&mut self, continuous: bool) {
        self.continuous = continuous;
    }

    /// Check if continuous mode
    pub fn is_continuous(&self) -> bool {
        self.continuous
    }

    /// STUB: Simulate transcription result
    /// In L12.5, this will be called by STT callback
    pub fn on_transcription(&mut self, text: impl Into<String>, confidence: f32) {
        self.transcript = text.into();
        self.confidence = confidence;
        self.state = VoiceState::Complete;
        
        log::info!("Voice transcription: '{}' (confidence: {})", self.transcript, confidence);
    }

    /// STUB: Simulate error
    pub fn on_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.state = VoiceState::Error;
        
        log::error!("Voice input error: {:?}", self.error);
    }

    /// Get error message
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Reset voice input state
    pub fn reset(&mut self) {
        self.state = VoiceState::Inactive;
        self.transcript.clear();
        self.confidence = 0.0;
        self.error = None;
    }
}

impl Default for VoiceInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Voice input errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum VoiceError {
    #[error("Microphone not available")]
    MicrophoneNotAvailable,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("STT service unavailable")]
    SttServiceUnavailable,
    
    #[error("Recognition failed: {0}")]
    RecognitionFailed(String),
}

/// Voice configuration for L12.5 integration
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Enable voice input
    pub enabled: bool,
    /// Language code
    pub language: String,
    /// Continuous listening
    pub continuous: bool,
    /// Auto-submit after silence
    pub auto_submit: bool,
    /// Silence timeout (ms)
    pub silence_timeout_ms: u32,
}

impl VoiceConfig {
    /// Create default config
    pub fn new() -> Self {
        Self {
            enabled: false,
            language: "en-US".to_string(),
            continuous: false,
            auto_submit: true,
            silence_timeout_ms: 2000,
        }
    }

    /// Enable voice input
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Voice UI element info
#[derive(Debug, Clone)]
pub struct VoiceUiElement {
    /// Microphone button position
    pub mic_button_rect: (u32, u32, u32, u32),
    /// Waveform display area
    pub waveform_rect: (u32, u32, u32, u32),
    /// Status text position
    pub status_position: (u32, u32),
}

impl VoiceUiElement {
    /// Calculate layout for address bar integration
    pub fn for_address_bar(address_bar_rect: (u32, u32, u32, u32)) -> Self {
        let (x, y, width, height) = address_bar_rect;
        
        // Microphone button at right end of address bar
        let mic_size = height - 8;
        let mic_x = x + width - mic_size - 4;
        let mic_y = y + 4;
        
        Self {
            mic_button_rect: (mic_x, mic_y, mic_size, mic_size),
            waveform_rect: (x + 40, y + height + 5, width - 80, 40),
            status_position: (x + 10, y + height + 55),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_input_creation() {
        let voice = VoiceInput::new();
        assert_eq!(voice.state(), VoiceState::Inactive);
        assert!(!voice.is_listening());
        assert_eq!(voice.language(), "en-US");
    }

    #[test]
    fn test_voice_input_start_stop() {
        let mut voice = VoiceInput::new();
        
        voice.start().unwrap();
        assert!(voice.is_listening());
        assert_eq!(voice.state(), VoiceState::Listening);
        
        voice.stop();
        assert_eq!(voice.state(), VoiceState::Processing);
    }

    #[test]
    fn test_voice_transcription() {
        let mut voice = VoiceInput::new();
        
        voice.start().unwrap();
        voice.on_transcription("hello world", 0.95);
        
        assert_eq!(voice.state(), VoiceState::Complete);
        assert_eq!(voice.transcript(), "hello world");
        assert_eq!(voice.confidence(), 0.95);
    }

    #[test]
    fn test_voice_error() {
        let mut voice = VoiceInput::new();
        
        voice.start().unwrap();
        voice.on_error("Microphone not found");
        
        assert_eq!(voice.state(), VoiceState::Error);
        assert_eq!(voice.error(), Some("Microphone not found"));
    }

    #[test]
    fn test_voice_ui_element() {
        let address_bar = (16, 16, 992, 40);
        let voice_ui = VoiceUiElement::for_address_bar(address_bar);
        
        assert_eq!(voice_ui.mic_button_rect.1, 20); // y position
        assert_eq!(voice_ui.mic_button_rect.3, 32); // height
    }

    #[test]
    fn test_voice_config() {
        let config = VoiceConfig::enabled();
        assert!(config.enabled);
        assert_eq!(config.language, "en-US");
        assert!(config.auto_submit);
    }
}
