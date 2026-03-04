//! Permission system

use std::collections::HashMap;

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Camera,
    Microphone,
    Location,
    Notifications,
}

/// Permission manager per origin
#[derive(Debug, Default)]
pub struct PermissionManager {
    permissions: HashMap<String, HashMap<Permission, PermissionState>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Prompt,
    Allow,
    Deny,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }
}
