use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// Represents errors that can occur when handling colors in Windows.
#[derive(Debug)]
pub enum WinColorError {
    /// Error when the gradient coordinates are invalid.
    InvalidGradientCoordinates(String),
    /// Error when the Windows accent color cannot be found.
    AccentColorNotFound,
    /// Error when the provided hex color format is invalid.
    InvalidHex(String),
    /// Error when the provided RGB color format is invalid.
    InvalidRgb(String),
    /// Error when the darken operation encounters an invalid input.
    InvalidDarken(String),
    /// Error when the lighten operation encounters an invalid input.
    InvalidLighten(String),
}

impl Display for WinColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WinColorError::InvalidGradientCoordinates(msg) => {
                write!(f, "Invalid gradient coordinates: {}", msg)
            }
            WinColorError::AccentColorNotFound => {
                write!(f, "Windows accent color not found")
            }
            WinColorError::InvalidHex(hex) => {
                write!(f, "Invalid hex format: {}", hex)
            }
            WinColorError::InvalidRgb(rgb) => {
                write!(f, "Invalid RGB format: {}", rgb)
            }
            WinColorError::InvalidDarken(darken) => {
                write!(f, "Invalid darken format: {}", darken)
            }
            WinColorError::InvalidLighten(lighten) => {
                write!(f, "Invalid lighten format: {}", lighten)
            }
        }
    }
}

impl Error for WinColorError {}
