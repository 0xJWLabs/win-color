//! This module defines color structures and utilities for managing colors
//! in a Windows environment using Direct2D.
//!
//! It provides functionality for both solid colors and gradients.
mod error;
mod gradient;
mod parser;
mod solid;
mod utils;

use parser::parse_color;
use parser::parse_color_mapping;
use serde::Deserialize;
use windows::core::Result as WinResult;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;
use windows::Win32::Graphics::Direct2D::ID2D1HwndRenderTarget;
use windows::Win32::Graphics::Direct2D::D2D1_BRUSH_PROPERTIES;
use windows::Win32::Graphics::Direct2D::D2D1_EXTEND_MODE_CLAMP;
use windows::Win32::Graphics::Direct2D::D2D1_GAMMA_2_2;
use windows::Win32::Graphics::Direct2D::D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES;

pub use error::WinColorError;
pub use gradient::ColorMapping;
pub use gradient::ColorMappingImpl;
pub use gradient::Gradient;
pub use gradient::GradientCoordinates;
pub use gradient::GradientDirection;
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
    Mapping(ColorMapping),
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
    /// Tries to create a `Color` from a global color definition.
    fn fetch(color: &GlobalColor, is_active: Option<bool>) -> Result<Color, WinColorError>;

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
    fn fetch(
        color_definition: &GlobalColor,
        is_active: Option<bool>,
    ) -> Result<Self, WinColorError> {
        match color_definition {
            GlobalColor::String(s) => parse_color(s.as_str(), is_active),
            GlobalColor::Mapping(gradient_def) => {
                parse_color_mapping(gradient_def.clone(), is_active)
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
