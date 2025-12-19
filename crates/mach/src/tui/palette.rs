#![allow(dead_code)]
use ratatui::style::Color;

// Text
pub const TEXT: Color = Color::Reset;
pub const TEXT_DIM: Color = Color::DarkGray;

// States (hierarchy: ACCENT > ACTIVE > FOCUS)
pub const FOCUS: Color = Color::LightBlue;
pub const ACTIVE: Color = Color::Yellow;
pub const ACCENT: Color = Color::Magenta;

// Chrome
pub const BORDER: Color = Color::DarkGray;

// Feedback
pub const ERROR: Color = Color::Red;
