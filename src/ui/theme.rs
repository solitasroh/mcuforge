use iocraft::prelude::*;

/// embtool color palette — minimal, modern
pub struct Theme;

impl Theme {
    // Primary accent (cyan/teal family)
    pub const ACCENT: Color = Color::Cyan;
    // Success
    pub const SUCCESS: Color = Color::Green;
    // Warning
    pub const WARNING: Color = Color::Yellow;
    // Error
    pub const ERROR: Color = Color::Red;
    // Muted/dim
    pub const MUTED: Color = Color::DarkGrey;
    // Info labels
    #[allow(dead_code)]
    pub const LABEL: Color = Color::Blue;
    // Border
    pub const BORDER: Color = Color::DarkGrey;
}
