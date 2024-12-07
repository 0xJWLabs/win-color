//! This module defines color structures and utilities for managing colors
//! in a Windows environment using Direct2D.
//!
//! It provides functionality for both solid colors and gradients.
mod constant;
mod error;
mod gradient;
mod solid;
mod utils;

use constant::COLOR_REGEX;
use constant::DARKEN_LIGHTEN_REGEX;
use constant::NAMED_COLORS;
use serde::Deserialize;
use utils::darken;
use utils::is_valid_direction;
use utils::lighten;
use utils::strip_string;
use windows::core::Result as WinResult;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::FALSE;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP;
use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;
use windows::Win32::Graphics::Direct2D::ID2D1HwndRenderTarget;
use windows::Win32::Graphics::Direct2D::D2D1_BRUSH_PROPERTIES;
use windows::Win32::Graphics::Direct2D::D2D1_EXTEND_MODE_CLAMP;
use windows::Win32::Graphics::Direct2D::D2D1_GAMMA_2_2;
use windows::Win32::Graphics::Direct2D::D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES;
use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;

pub use error::WinColorError;
pub use gradient::Gradient;
pub use gradient::GradientCoordinates;
pub use gradient::GradientDirection;
pub use gradient::GradientMapping;
pub use solid::Solid;

/// The `Color` enum represents both solid colors and gradients.
/// It can either be a solid color or a gradient.
#[derive(Debug, Clone)]
pub enum Color {
    /// Represents a solid color.
    Solid(Solid),
    /// Represents a gradient color.
    Gradient(Gradient),
}

/// The `GlobalColor` enum represents a global color, which can either be
/// a string (hex color or name) or a gradient mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum GlobalColor {
    /// A string representing a color.
    String(String),
    /// A gradient mapping defining a gradient color.
    Mapping(GradientMapping),
}

impl Default for GlobalColor {
    fn default() -> Self {
        // The default color is black (#000000).
        Self::String("#000000".to_string())
    }
}

/// The `ColorImpl` trait defines methods for working with colors, including
/// converting colors from strings, mapping, and global color definitions,
/// as well as handling opacity and rendering to Direct2D brushes.
pub trait ColorImpl {
    /// Tries to create a `Color` from a string.
    fn try_from_string(color: &str, is_active: Option<bool>) -> Result<Color, WinColorError>;

    /// Tries to create a `Color` from a gradient mapping.
    fn try_from_mapping(
        color: GradientMapping,
        is_active: Option<bool>,
    ) -> Result<Color, WinColorError>;

    /// Tries to create a `Color` from a global color definition.
    fn try_from_global(
        color: &GlobalColor,
        is_active: Option<bool>,
    ) -> Result<Color, WinColorError>;

    /// Sets the opacity of the color.
    fn set_opacity(&mut self, opacity: f32);

    /// Gets the opacity of the color.
    fn get_opacity(&self) -> f32;

    /// Converts the color to a Direct2D brush.
    fn to_d2d1_brush(
        &self,
        render_target: &ID2D1HwndRenderTarget,
        window_rect: &RECT,
        brush_properties: &D2D1_BRUSH_PROPERTIES,
    ) -> WinResult<ID2D1Brush>;
}

impl ColorImpl for Color {
    fn try_from_string(color: &str, is_active: Option<bool>) -> Result<Self, WinColorError> {
        if color.starts_with("gradient(") && color.ends_with(")") {
            return Self::try_from_string(
                strip_string(color.to_string(), &["gradient("], ')').as_str(),
                is_active,
            );
        }
        let color_re = &COLOR_REGEX;

        let color_matches: Vec<&str> = color_re
            .captures_iter(color)
            .filter_map(|cap| cap.get(0).map(|m| m.as_str()))
            .collect();

        if color_matches.len() == 1 {
            let color = color_matches[0].to_string().to_d2d1_color(is_active)?;

            return Ok(Self::Solid(Solid {
                color,
                opacity: 0.0,
            }));
        }

        let remaining_input = color[color.rfind(color_matches.last().unwrap()).unwrap()
            + color_matches.last().unwrap().len()..]
            .trim_start();

        let remaining_input_arr: Vec<&str> = remaining_input
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                (!trimmed.is_empty()).then_some(trimmed)
            })
            .collect();

        let direction = remaining_input_arr
            .iter()
            .find(|&&input| is_valid_direction(input))
            .map(|&s| s.to_string())
            .unwrap_or_else(|| "to_right".to_string());

        let colors: Vec<D2D1_COLOR_F> = color_matches
            .iter()
            .filter_map(|&color| color.to_string().to_d2d1_color(is_active).ok()) // Only keep Ok values
            .collect();

        let num_colors = colors.len();
        let step = 1.0 / (num_colors - 1) as f32;

        let gradient_stops = colors
            .into_iter()
            .enumerate()
            .map(|(i, color)| D2D1_GRADIENT_STOP {
                position: i as f32 * step,
                color,
            })
            .collect();

        let direction = GradientCoordinates::try_from(direction.as_str())?;

        Ok(Self::Gradient(Gradient {
            gradient_stops,
            direction,
            opacity: 0.0,
        }))
    }

    fn try_from_mapping(
        color: GradientMapping,
        is_active: Option<bool>,
    ) -> Result<Self, WinColorError> {
        match color.colors.len() {
            0 => Ok(Color::Solid(Solid {
                color: D2D1_COLOR_F::default(),
                opacity: 0.0,
            })),
            1 => Ok(Color::Solid(Solid {
                color: color.colors[0].clone().to_d2d1_color(is_active)?,
                opacity: 0.0,
            })),
            _ => {
                let num_colors = color.colors.len();
                let step = 1.0 / (num_colors - 1) as f32;
                let gradient_stops: Vec<D2D1_GRADIENT_STOP> = color
                    .colors
                    .iter()
                    .enumerate()
                    .filter_map(|(i, hex)| {
                        hex.to_string()
                            .to_d2d1_color(is_active)
                            .ok() // This will convert the Result to Option, ignoring errors
                            .map(|color| D2D1_GRADIENT_STOP {
                                position: i as f32 * step,
                                color,
                            })
                    })
                    .collect(); // Collect the successful results into a Vec

                let direction = match color.direction {
                    GradientDirection::Direction(direction) => {
                        GradientCoordinates::try_from(direction.as_str())
                    }
                    GradientDirection::Coordinates(direction) => Ok(direction),
                }?;

                Ok(Color::Gradient(Gradient {
                    gradient_stops,
                    direction,
                    opacity: 0.0,
                }))
            }
        }
    }

    fn try_from_global(
        color_definition: &GlobalColor,
        is_active: Option<bool>,
    ) -> Result<Self, WinColorError> {
        match color_definition {
            GlobalColor::String(s) => Self::try_from_string(s.as_str(), is_active),
            GlobalColor::Mapping(gradient_def) => {
                Self::try_from_mapping(gradient_def.clone(), is_active)
            }
        }
    }

    fn set_opacity(&mut self, opacity: f32) {
        match self {
            Color::Gradient(gradient) => gradient.opacity = opacity,
            Color::Solid(solid) => solid.opacity = opacity,
        }
    }

    fn get_opacity(&self) -> f32 {
        match self {
            Color::Gradient(gradient) => gradient.opacity,
            Color::Solid(solid) => solid.opacity,
        }
    }

    fn to_d2d1_brush(
        &self,
        render_target: &ID2D1HwndRenderTarget,
        window_rect: &RECT,
        brush_properties: &D2D1_BRUSH_PROPERTIES,
    ) -> WinResult<ID2D1Brush> {
        match self {
            Color::Solid(solid) => unsafe {
                let brush =
                    render_target.CreateSolidColorBrush(&solid.color, Some(brush_properties))?;

                brush.SetOpacity(solid.opacity);

                Ok(brush.into())
            },
            Color::Gradient(gradient) => unsafe {
                let width = (window_rect.right - window_rect.left) as f32;
                let height = (window_rect.bottom - window_rect.top) as f32;

                let gradient_properties = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                    startPoint: D2D_POINT_2F {
                        x: gradient.direction.start[0] * width,
                        y: gradient.direction.start[1] * height,
                    },
                    endPoint: D2D_POINT_2F {
                        x: gradient.direction.end[0] * width,
                        y: gradient.direction.end[1] * height,
                    },
                };

                let Ok(gradient_stop_collection) = render_target.CreateGradientStopCollection(
                    &gradient.gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                ) else {
                    // TODO instead of panicking, I should just return a default value
                    panic!("could not create gradient_stop_collection!");
                };

                let brush = render_target.CreateLinearGradientBrush(
                    &gradient_properties,
                    Some(brush_properties),
                    &gradient_stop_collection,
                )?;

                brush.SetOpacity(gradient.opacity);

                Ok(brush.into())
            },
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Solid(Solid {
            color: D2D1_COLOR_F::default(),
            opacity: 0.0,
        })
    }
}

/// A trait for converting various types to a `D2D1_COLOR_F` (Direct2D color representation).
pub trait ToColor {
    /// Converts a string or other type to a `D2D1_COLOR_F`.
    fn to_d2d1_color(self, is_active: Option<bool>) -> Result<D2D1_COLOR_F, WinColorError>;
}

impl ToColor for String {
    fn to_d2d1_color(self, is_active_color: Option<bool>) -> Result<D2D1_COLOR_F, WinColorError> {
        if self == "accent" {
            let mut pcr_colorization: u32 = 0;
            let mut pf_opaqueblend: BOOL = FALSE;

            if unsafe { DwmGetColorizationColor(&mut pcr_colorization, &mut pf_opaqueblend) }
                .is_err()
            {
                return Err(WinColorError::AccentColorNotFound);
            }

            let r = ((pcr_colorization & 0x00FF0000) >> 16) as f32 / 255.0;
            let g = ((pcr_colorization & 0x0000FF00) >> 8) as f32 / 255.0;
            let b = (pcr_colorization & 0x000000FF) as f32 / 255.0;
            let avg = (r + g + b) / 3.0;

            return match is_active_color {
                Some(true) => Ok(D2D1_COLOR_F { r, g, b, a: 1.0 }),
                _ => Ok(D2D1_COLOR_F {
                    r: avg / 1.5 + r / 10.0,
                    g: avg / 1.5 + g / 10.0,
                    b: avg / 1.5 + b / 10.0,
                    a: 1.0,
                }),
            };
        } else if self.starts_with("#") {
            if self.len() != 7 && self.len() != 9 && self.len() != 4 && self.len() != 5 {
                return Err(WinColorError::InvalidHex(self));
            }

            let hex = match self.len() {
                4 | 5 => format!(
                    "#{}{}{}{}",
                    self.get(1..2).unwrap_or("").repeat(2),
                    self.get(2..3).unwrap_or("").repeat(2),
                    self.get(3..4).unwrap_or("").repeat(2),
                    self.get(4..5).unwrap_or("").repeat(2)
                ),
                _ => self.to_string(),
            };

            // Parse RGB and Alpha
            let (r, g, b, a) = (
                u8::from_str_radix(&hex[1..3], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex[3..5], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex[5..7], 16).unwrap_or(0) as f32 / 255.0,
                if hex.len() == 9 {
                    u8::from_str_radix(&hex[7..9], 16).unwrap_or(0) as f32 / 255.0
                } else {
                    1.0
                },
            );

            return Ok(D2D1_COLOR_F { r, g, b, a });
        } else if self.starts_with("rgb(") || self.starts_with("rgba(") {
            let rgba = strip_string(self.clone(), &["rgb(", "rgba("], ')');
            let components: Vec<&str> = rgba.split(',').map(|s| s.trim()).collect();

            if components.len() != 3 && components.len() != 4 {
                return Err(WinColorError::InvalidRgb(self.clone()));
            }

            let r: f32 = components[0].parse::<u32>().unwrap_or(0) as f32 / 255.0;
            let g: f32 = components[1].parse::<u32>().unwrap_or(0) as f32 / 255.0;
            let b: f32 = components[2].parse::<u32>().unwrap_or(0) as f32 / 255.0;
            let a = components
                .get(3)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);

            return Ok(D2D1_COLOR_F { r, g, b, a });
        } else if self.starts_with("darken(") || self.starts_with("lighten(") {
            let darken_lighten_re = &DARKEN_LIGHTEN_REGEX;

            if let Some(caps) = darken_lighten_re.captures(self.as_str()) {
                if caps.len() != 4 {
                    if self.starts_with("darken(") {
                        return Err(WinColorError::InvalidDarken(self));
                    }
                    return Err(WinColorError::InvalidLighten(self));
                }
                let dark_or_lighten = &caps[1];
                let color_str = &caps[2];
                let percentage = &caps[3].parse::<f32>().unwrap_or(10.0);

                let color = color_str.to_string().to_d2d1_color(is_active_color)?;
                let color_res = match dark_or_lighten {
                    "darken" => darken(color, *percentage),
                    "lighten" => lighten(color, *percentage),
                    _ => color,
                };

                return Ok(color_res);
            }

            if self.starts_with("darken(") {
                return Err(WinColorError::InvalidDarken(self));
            }
            return Err(WinColorError::InvalidLighten(self));
        } else if let Some(color) = NAMED_COLORS.get(self.as_str()) {
            return Ok(*color);
        }

        Ok(D2D1_COLOR_F::default())
    }
}
