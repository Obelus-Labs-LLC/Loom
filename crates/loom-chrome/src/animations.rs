//! Physics-based animations

/// Spring animation configuration
#[derive(Debug, Clone)]
pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            stiffness: 200.0,
            damping: 25.0,
            mass: 1.0,
        }
    }
}

/// Thread pulse animation
#[derive(Debug)]
pub struct ThreadPulse {
    pub active: bool,
    pub intensity: f32,
}

impl ThreadPulse {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.0,
        }
    }
}
