//! Permission System and UI
//!
//! Phase L15: Security & Sandboxing
//! - Permission types (camera, mic, location, notifications)
//! - Permission state management per origin
//! - Permission UI/dialog framework
//! - Permission persistence

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::HashMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Camera access
    Camera,
    /// Microphone access
    Microphone,
    /// Geolocation
    Location,
    /// Desktop notifications
    Notifications,
    /// Persistent storage
    PersistentStorage,
    /// Clipboard read/write
    Clipboard,
    /// MIDI access
    Midi,
    /// Bluetooth access
    Bluetooth,
    /// USB access
    Usb,
    /// Serial port access
    Serial,
    /// HID access
    Hid,
    /// Push notifications
    Push,
    /// Background sync
    BackgroundSync,
    /// Payment handler
    Payment,
}

impl Permission {
    /// Get permission name
    pub fn name(&self) -> &'static str {
        match self {
            Permission::Camera => "camera",
            Permission::Microphone => "microphone",
            Permission::Location => "geolocation",
            Permission::Notifications => "notifications",
            Permission::PersistentStorage => "persistent-storage",
            Permission::Clipboard => "clipboard",
            Permission::Midi => "midi",
            Permission::Bluetooth => "bluetooth",
            Permission::Usb => "usb",
            Permission::Serial => "serial",
            Permission::Hid => "hid",
            Permission::Push => "push",
            Permission::BackgroundSync => "background-sync",
            Permission::Payment => "payment",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Permission::Camera => "Use your camera",
            Permission::Microphone => "Use your microphone",
            Permission::Location => "Know your location",
            Permission::Notifications => "Send notifications",
            Permission::PersistentStorage => "Store data persistently",
            Permission::Clipboard => "Read from and write to the clipboard",
            Permission::Midi => "Access MIDI devices",
            Permission::Bluetooth => "Connect to Bluetooth devices",
            Permission::Usb => "Connect to USB devices",
            Permission::Serial => "Connect to serial ports",
            Permission::Hid => "Access human interface devices",
            Permission::Push => "Receive push notifications",
            Permission::BackgroundSync => "Sync in the background",
            Permission::Payment => "Handle payments",
        }
    }

    /// Get icon/emoji for permission
    pub fn icon(&self) -> &'static str {
        match self {
            Permission::Camera => "📷",
            Permission::Microphone => "🎤",
            Permission::Location => "📍",
            Permission::Notifications => "🔔",
            Permission::PersistentStorage => "💾",
            Permission::Clipboard => "📋",
            Permission::Midi => "🎹",
            Permission::Bluetooth => "📡",
            Permission::Usb => "🔌",
            Permission::Serial => "🔌",
            Permission::Hid => "🎮",
            Permission::Push => "📲",
            Permission::BackgroundSync => "🔄",
            Permission::Payment => "💳",
        }
    }

    /// Check if this permission is considered sensitive
    pub fn is_sensitive(&self) -> bool {
        matches!(self,
            Permission::Camera |
            Permission::Microphone |
            Permission::Location |
            Permission::Notifications
        )
    }
}

/// Permission states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// User hasn't been asked yet
    Prompt,
    /// User granted permission
    Granted,
    /// User denied permission
    Denied,
}

impl PermissionState {
    /// Check if permission is granted
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionState::Granted)
    }

    /// Check if permission is denied
    pub fn is_denied(&self) -> bool {
        matches!(self, PermissionState::Denied)
    }
}

/// Permission request details
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// Origin requesting permission
    pub origin: String,
    /// Permission being requested
    pub permission: Permission,
    /// Whether this was triggered by user gesture
    pub user_gesture: bool,
    /// Optional context about why permission is needed
    pub context: Option<String>,
}

/// Permission dialog response
#[derive(Debug, Clone)]
pub enum PermissionResponse {
    /// Grant permission
    Grant,
    /// Deny permission
    Deny,
    /// Dismiss without deciding (treat as deny)
    Dismiss,
}

/// Permission manager per origin
#[derive(Debug, Default)]
pub struct PermissionManager {
    /// Origin -> (Permission -> State)
    permissions: HashMap<String, HashMap<Permission, PermissionState>>,
    /// Temporary grants that expire at end of session
    session_grants: HashMap<(String, Permission), ()>,
}

impl PermissionManager {
    /// Create new permission manager
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            session_grants: HashMap::new(),
        }
    }

    /// Get permission state for an origin
    pub fn get_state(&self, origin: &str, permission: Permission) -> PermissionState {
        self.permissions
            .get(origin)
            .and_then(|p| p.get(&permission))
            .copied()
            .unwrap_or(PermissionState::Prompt)
    }

    /// Set permission state for an origin
    pub fn set_state(&mut self, origin: &str, permission: Permission, state: PermissionState) {
        self.permissions
            .entry(origin.to_string())
            .or_default()
            .insert(permission, state);
        
        // Remove from session grants if setting to denied
        if matches!(state, PermissionState::Denied) {
            self.session_grants.remove(&(origin.to_string(), permission));
        }
    }

    /// Grant permission permanently
    pub fn grant(&mut self, origin: &str, permission: Permission) {
        self.set_state(origin, permission, PermissionState::Granted);
    }

    /// Deny permission
    pub fn deny(&mut self, origin: &str, permission: Permission) {
        self.set_state(origin, permission, PermissionState::Denied);
    }

    /// Grant permission for this session only
    pub fn grant_session(&mut self, origin: &str, permission: Permission) {
        self.session_grants.insert((origin.to_string(), permission), ());
    }

    /// Check if permission is granted (including session grants)
    pub fn is_granted(&self, origin: &str, permission: Permission) -> bool {
        // Check permanent grants
        if self.get_state(origin, permission).is_granted() {
            return true;
        }
        
        // Check session grants
        self.session_grants.contains_key(&(origin.to_string(), permission))
    }

    /// Check if permission should prompt
    pub fn should_prompt(&self, origin: &str, permission: Permission) -> bool {
        matches!(self.get_state(origin, permission), PermissionState::Prompt) &&
        !self.session_grants.contains_key(&(origin.to_string(), permission))
    }

    /// Revoke permission
    pub fn revoke(&mut self, origin: &str, permission: Permission) {
        self.permissions
            .entry(origin.to_string())
            .or_default()
            .remove(&permission);
        self.session_grants.remove(&(origin.to_string(), permission));
    }

    /// Revoke all permissions for an origin
    pub fn revoke_all(&mut self, origin: &str) {
        self.permissions.remove(origin);
        
        // Remove all session grants for this origin
        let to_remove: Vec<_> = self.session_grants
            .keys()
            .filter(|(o, _)| o == origin)
            .cloned()
            .collect();
        
        for key in to_remove {
            self.session_grants.remove(&key);
        }
    }

    /// Clear all session grants (call when session ends)
    pub fn clear_session_grants(&mut self) {
        self.session_grants.clear();
    }

    /// Get all permissions for an origin
    pub fn get_origin_permissions(&self, origin: &str) -> Vec<(Permission, PermissionState)> {
        self.permissions
            .get(origin)
            .map(|p| p.iter().map(|(&k, &v)| (k, v)).collect())
            .unwrap_or_default()
    }

    /// Get origins with permissions
    pub fn get_origins(&self) -> Vec<String> {
        self.permissions.keys().cloned().collect()
    }
}

/// Permission UI dialog
#[derive(Debug)]
pub struct PermissionDialog {
    /// Current request being shown
    pub current_request: Option<PermissionRequest>,
    /// Whether dialog is visible
    pub visible: bool,
    /// Remember choice checkbox state
    pub remember_choice: bool,
}

impl PermissionDialog {
    /// Create new dialog
    pub fn new() -> Self {
        Self {
            current_request: None,
            visible: false,
            remember_choice: false,
        }
    }

    /// Show permission request
    pub fn show_request(&mut self, request: PermissionRequest) {
        self.current_request = Some(request);
        self.visible = true;
        self.remember_choice = false;
    }

    /// Hide dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.current_request = None;
    }

    /// Handle user response
    pub fn handle_response(
        &mut self,
        response: PermissionResponse,
        manager: &mut PermissionManager,
    ) {
        if let Some(ref request) = self.current_request {
            match response {
                PermissionResponse::Grant => {
                    if self.remember_choice {
                        manager.grant(&request.origin, request.permission);
                    } else {
                        manager.grant_session(&request.origin, request.permission);
                    }
                }
                PermissionResponse::Deny | PermissionResponse::Dismiss => {
                    if self.remember_choice {
                        manager.deny(&request.origin, request.permission);
                    }
                    // If not remembering, just don't grant
                }
            }
        }
        
        self.hide();
    }

    /// Get dialog title
    pub fn title(&self) -> String {
        if let Some(ref request) = self.current_request {
            format!("{} wants to access your {}", 
                request.origin, 
                request.permission.name()
            )
        } else {
            String::new()
        }
    }

    /// Get dialog message
    pub fn message(&self) -> String {
        if let Some(ref request) = self.current_request {
            let mut msg = format!("{} wants to {}",
                request.origin,
                request.permission.description()
            );
            
            if let Some(ref context) = request.context {
                msg.push_str("\n\nReason: ");
                msg.push_str(context);
            }
            
            msg
        } else {
            String::new()
        }
    }

    /// Check if a permission is currently being requested
    pub fn is_requesting(&self, origin: &str, permission: Permission) -> bool {
        self.visible && self.current_request.as_ref()
            .map(|r| r.origin == origin && r.permission == permission)
            .unwrap_or(false)
    }
}

impl Default for PermissionDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission prompt UI element info
#[derive(Debug, Clone)]
pub struct PermissionUiElement {
    /// Element type (icon, text, button, checkbox)
    pub element_type: UiElementType,
    /// Content/text
    pub content: String,
    /// Position (x, y)
    pub position: (u32, u32),
    /// Size (width, height)
    pub size: (u32, u32),
    /// Action when clicked
    pub action: Option<UiAction>,
}

/// UI element types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiElementType {
    Icon,
    Text,
    PrimaryButton,
    SecondaryButton,
    Checkbox,
}

/// UI actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    Grant,
    Deny,
    ToggleRemember,
}

/// Generate UI elements for permission dialog
pub fn generate_permission_ui(
    dialog: &PermissionDialog,
    area: (u32, u32, u32, u32), // x, y, width, height
) -> Vec<PermissionUiElement> {
    let mut elements = Vec::new();
    
    if dialog.current_request.is_none() {
        return elements;
    }
    
    let (x, y, width, _height) = area;
    let padding = 20u32;
    
    // Permission icon
    if let Some(ref request) = dialog.current_request {
        elements.push(PermissionUiElement {
            element_type: UiElementType::Icon,
            content: request.permission.icon().to_string(),
            position: (x + width / 2 - 32, y + padding),
            size: (64, 64),
            action: None,
        });
        
        // Title
        elements.push(PermissionUiElement {
            element_type: UiElementType::Text,
            content: dialog.title(),
            position: (x + padding, y + padding + 80),
            size: (width - 2 * padding, 30),
            action: None,
        });
        
        // Message
        elements.push(PermissionUiElement {
            element_type: UiElementType::Text,
            content: dialog.message(),
            position: (x + padding, y + padding + 120),
            size: (width - 2 * padding, 80),
            action: None,
        });
        
        // Remember checkbox
        elements.push(PermissionUiElement {
            element_type: UiElementType::Checkbox,
            content: "Remember my decision".to_string(),
            position: (x + padding, y + padding + 220),
            size: (200, 24),
            action: Some(UiAction::ToggleRemember),
        });
        
        // Deny button
        elements.push(PermissionUiElement {
            element_type: UiElementType::SecondaryButton,
            content: "Block".to_string(),
            position: (x + width - 2 * padding - 160, y + padding + 260),
            size: (70, 36),
            action: Some(UiAction::Deny),
        });
        
        // Grant button
        elements.push(PermissionUiElement {
            element_type: UiElementType::PrimaryButton,
            content: "Allow".to_string(),
            position: (x + width - padding - 80, y + padding + 260),
            size: (70, 36),
            action: Some(UiAction::Grant),
        });
    }
    
    elements
}

/// Security indicator for address bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityIndicator {
    /// Secure HTTPS connection
    Secure,
    /// Insecure HTTP connection
    Insecure,
    /// Mixed content (HTTPS page with HTTP resources)
    MixedContent,
    /// Invalid certificate
    InvalidCert,
    /// Local file
    Local,
    /// Internal page (about:, chrome:, etc.)
    Internal,
}

impl SecurityIndicator {
    /// Get icon for indicator
    pub fn icon(&self) -> &'static str {
        match self {
            SecurityIndicator::Secure => "🔒",
            SecurityIndicator::Insecure => "⚠️",
            SecurityIndicator::MixedContent => "⚠️",
            SecurityIndicator::InvalidCert => "🚫",
            SecurityIndicator::Local => "📁",
            SecurityIndicator::Internal => "⚙️",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            SecurityIndicator::Secure => "Secure connection",
            SecurityIndicator::Insecure => "Not secure",
            SecurityIndicator::MixedContent => "Mixed content warning",
            SecurityIndicator::InvalidCert => "Certificate error",
            SecurityIndicator::Local => "Local file",
            SecurityIndicator::Internal => "Internal page",
        }
    }

    /// Get color (as hex for UI)
    pub fn color(&self) -> u32 {
        match self {
            SecurityIndicator::Secure => 0xFF4CAF50, // Green
            SecurityIndicator::Insecure => 0xFFFFA726, // Orange
            SecurityIndicator::MixedContent => 0xFFFFA726, // Orange
            SecurityIndicator::InvalidCert => 0xFFE53935, // Red
            SecurityIndicator::Local => 0xFF42A5F5, // Blue
            SecurityIndicator::Internal => 0xFF9E9E9E, // Gray
        }
    }
}

/// Determine security indicator from URL and load info
pub fn get_security_indicator(url: &str, has_cert_errors: bool, has_mixed_content: bool) -> SecurityIndicator {
    if url.starts_with("https://") {
        if has_cert_errors {
            SecurityIndicator::InvalidCert
        } else if has_mixed_content {
            SecurityIndicator::MixedContent
        } else {
            SecurityIndicator::Secure
        }
    } else if url.starts_with("http://") {
        SecurityIndicator::Insecure
    } else if url.starts_with("file://") {
        SecurityIndicator::Local
    } else if url.starts_with("about:") || url.starts_with("chrome://") {
        SecurityIndicator::Internal
    } else {
        SecurityIndicator::Insecure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_manager() {
        let mut manager = PermissionManager::new();
        
        // Initially should prompt
        assert!(manager.should_prompt("https://example.com", Permission::Camera));
        
        // Grant permission
        manager.grant("https://example.com", Permission::Camera);
        assert!(manager.is_granted("https://example.com", Permission::Camera));
        assert!(!manager.should_prompt("https://example.com", Permission::Camera));
        
        // Deny permission
        manager.deny("https://example.com", Permission::Microphone);
        assert!(!manager.is_granted("https://example.com", Permission::Microphone));
        
        // Revoke permission
        manager.revoke("https://example.com", Permission::Camera);
        assert!(!manager.is_granted("https://example.com", Permission::Camera));
    }

    #[test]
    fn test_session_grants() {
        let mut manager = PermissionManager::new();
        
        manager.grant_session("https://example.com", Permission::Location);
        assert!(manager.is_granted("https://example.com", Permission::Location));
        
        // After clearing session grants
        manager.clear_session_grants();
        assert!(!manager.is_granted("https://example.com", Permission::Location));
    }

    #[test]
    fn test_permission_dialog() {
        let mut dialog = PermissionDialog::new();
        let mut manager = PermissionManager::new();
        
        let request = PermissionRequest {
            origin: "https://example.com".to_string(),
            permission: Permission::Camera,
            user_gesture: true,
            context: None,
        };
        
        dialog.show_request(request);
        assert!(dialog.visible);
        
        // Grant permission
        dialog.handle_response(PermissionResponse::Grant, &mut manager);
        assert!(!dialog.visible);
        assert!(manager.is_granted("https://example.com", Permission::Camera));
    }

    #[test]
    fn test_security_indicator() {
        assert!(matches!(
            get_security_indicator("https://example.com", false, false),
            SecurityIndicator::Secure
        ));
        
        assert!(matches!(
            get_security_indicator("http://example.com", false, false),
            SecurityIndicator::Insecure
        ));
        
        assert!(matches!(
            get_security_indicator("https://example.com", true, false),
            SecurityIndicator::InvalidCert
        ));
        
        assert!(matches!(
            get_security_indicator("https://example.com", false, true),
            SecurityIndicator::MixedContent
        ));
    }

    #[test]
    fn test_permission_names() {
        assert_eq!(Permission::Camera.name(), "camera");
        assert_eq!(Permission::Microphone.name(), "microphone");
        assert_eq!(Permission::Location.name(), "geolocation");
    }

    #[test]
    fn test_permission_states() {
        assert!(PermissionState::Granted.is_granted());
        assert!(!PermissionState::Denied.is_granted());
        assert!(PermissionState::Denied.is_denied());
    }
}
