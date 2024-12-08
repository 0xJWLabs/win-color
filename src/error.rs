use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// Represents errors that can occur when handling colors in Windows.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WinColorError {
    /// Error when the gradient coordinates are invalid.
    InvalidGradientCoordinates(String),
    /// Error when the Windows accent color cannot be found.
    InvalidAccent,
    /// Error when the provided hex color format is invalid.
    InvalidHex(String),
    /// Error when the provided RGB color format is invalid.
    InvalidRgb(String),
    /// Error when the darken or lighten operation encounters an invalid input.
    InvalidDarkenOrLighten(String, bool),

    InvalidUnknown,
}

impl Display for WinColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::InvalidHex(hex) => f.write_str(format!("invalid hex format: {}", hex).as_str()),
            Self::InvalidRgb(rgb) => f.write_str(format!("invalid rgb format: {}", rgb).as_str()),
            Self::InvalidUnknown => f.write_str("invalid unknown format"),
            Self::InvalidAccent => f.write_str("invalid accent color: accent color not found"),
            Self::InvalidGradientCoordinates(coordinates) => {
                f.write_str(format!("invalid gradient coordinates: {}", coordinates).as_str())
            }
            Self::InvalidDarkenOrLighten(s, darken_or_lighten) => match darken_or_lighten {
                true => f.write_str(format!("invalid darken format: {}", s).as_str()),
                false => f.write_str(format!("invalid lighten format: {}", s).as_str()),
            },
        }
    }
}

impl Error for WinColorError {}
