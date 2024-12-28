//! This module defines structures and utilities for managing colors in a Windows environment using Direct2D.
//!
//! It includes functionality for handling both solid colors and gradients,
//! allowing the creation, manipulation, and rendering of color values with Direct2D brushes.
//!
//! The module provides:
//! - Representation of solid colors using `Solid` struct.
//! - Representation of gradient colors using the `Gradient` struct, including customizable direction and color stops.
//! - Enums like `Color` and `GlobalColor` to abstract different color types and their sources, such as strings or gradient mappings.
//! - Methods for converting these color types into Direct2D brushes for rendering, as well as handling opacity and transformations.
mod error;
mod gradient;
mod parser;
mod solid;
mod utils;

use parser::parse_color;
use parser::parse_color_mapping;
use serde::Deserialize;
use windows::core::Result as WinResult;
use windows::Foundation::Numerics::Matrix3x2;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;
use windows::Win32::Graphics::Direct2D::ID2D1HwndRenderTarget;
use windows::Win32::Graphics::Direct2D::D2D1_BRUSH_PROPERTIES;
use windows::Win32::Graphics::Direct2D::D2D1_EXTEND_MODE_CLAMP;
use windows::Win32::Graphics::Direct2D::D2D1_GAMMA_2_2;
use windows::Win32::Graphics::Direct2D::D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES;

pub use error::Error;
pub use error::ErrorKind;
pub use error::Result;
pub use gradient::ColorMapping;
pub use gradient::ColorMappingImpl;
pub use gradient::Gradient;
pub use gradient::GradientCoordinates;
pub use gradient::GradientDirection;
pub use gradient::GradientImpl;
pub use solid::Solid;

/// The `Color` enum represents different types of colors, including both solid colors and gradients.
/// It can be either a solid color or a gradient, allowing flexibility in color representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    /// Represents a solid color.
    ///
    /// A `Solid` variant stores a single color, typically represented by an RGBA value.
    Solid(Solid),
    /// Represents a gradient color.
    ///
    /// A `Gradient` variant stores a color defined by a gradient, which may involve multiple color stops
    /// and a direction (for linear gradients).
    Gradient(Gradient),
}

/// The `GlobalColor` enum represents a global color that can be either a color string (e.g., a hex color code or a color name)
/// or a mapping to a gradient definition.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalColor {
    /// A string representing a color, which could be a hex color code or a color name.
    ///
    /// This variant is used for basic color definitions like `"#FF5733"` or `"red"`.
    String(String),
    /// A mapping defining a gradient color.
    ///
    /// This variant is used when the color is a gradient and contains a `ColorMapping` to define the gradient's
    /// color stops, direction, and other properties.
    Mapping(ColorMapping),
}

impl Default for GlobalColor {
    fn default() -> Self {
        // The default global color is black (`#000000`).
        // This is the fallback color when no other color is specified.
        Self::String("#000000".to_string())
    }
}

/// The `ColorImpl` trait defines methods for manipulating colors, including
/// converting colors from global color definitions, setting and getting opacity,
/// rendering the color as a Direct2D brush, and applying transformations.
///
/// This trait provides functionality to work with colors, including retrieving
/// them from global color definitions, adjusting opacity, and rendering them
/// as Direct2D brushes. It also includes methods for applying transformations
/// and managing brush properties.
///
/// # Methods
/// - `fetch`: Tries to create a `Color` from a global color definition (`GlobalColor`).
///   Optionally, a boolean flag (`is_active`) can be provided to modify the behavior.
/// - `set_opacity`: Sets the opacity of the color, where `opacity` is a float value between 0.0 and 1.0.
/// - `get_opacity`: Retrieves the current opacity of the color, if available.
/// - `get_brush`: Returns an optional reference to the Direct2D brush associated with the color.
/// - `set_transform`: Applies a transformation matrix to the color.
/// - `to_d2d1_brush`: Converts the color to a Direct2D brush using the provided render target,
///   window rectangle, and brush properties.
pub trait ColorImpl {
    /// Attempts to create a `Color` from a global color definition.
    ///
    /// This method fetches the color corresponding to a `GlobalColor` definition.
    /// It optionally takes a boolean flag (`is_active`) that may influence how the color is fetched.
    ///
    /// # Parameters
    /// - `color`: A reference to the `GlobalColor` definition.
    /// - `is_active`: An optional boolean that may affect the fetched color (e.g., active or inactive state).
    ///
    /// # Returns
    /// A `Result` containing either the fetched `Color` or a `WinColorError` if the operation fails.
    fn from_global_color(color: &GlobalColor, is_active: Option<bool>) -> Result<Color>;

    /// Sets the opacity of the color.
    ///
    /// This method adjusts the opacity of the color to the specified `opacity` value.
    ///
    /// # Parameters
    /// - `opacity`: A floating-point value representing the opacity (from 0.0 for fully transparent to 1.0 for fully opaque).
    fn set_opacity(&self, opacity: f32);

    /// Retrieves the current opacity of the color, if available.
    ///
    /// This method returns an `Option<f32>`, where `Some(f32)` indicates the opacity value,
    /// and `None` means no opacity value is set.
    ///
    /// # Returns
    /// An `Option<f32>` containing the opacity value or `None` if no opacity is set.
    fn get_opacity(&self) -> Option<f32>;

    /// Retrieves the brush associated with this color, if available.
    ///
    /// This method returns an optional reference to the `ID2D1Brush` that represents the color.
    ///
    /// # Returns
    /// An `Option<&ID2D1Brush>`, which is `Some` if the brush is available, or `None` if it isn't.
    fn get_brush(&self) -> Option<&ID2D1Brush>;

    /// Applies a transformation matrix to the color.
    ///
    /// This method sets a transformation (e.g., scaling, rotation) on the color using the provided
    /// transformation matrix.
    ///
    /// # Parameters
    /// - `transform`: A reference to the `Matrix3x2` transformation matrix that will be applied.
    fn set_transform(&self, transform: &Matrix3x2);

    /// Converts the color to a Direct2D brush.
    ///
    /// This method creates a Direct2D brush (`ID2D1Brush`) from the color, which can be used for rendering
    /// on a Direct2D render target. The brush is initialized with the given window rectangle and brush properties.
    ///
    /// # Parameters
    /// - `render_target`: The Direct2D render target on which the brush will be applied.
    /// - `window_rect`: The dimensions of the window, used to adjust the brush's rendering.
    /// - `brush_properties`: The properties that define how the brush will behave.
    ///
    /// # Returns
    /// A `WinResult<()>`, indicating success or failure.
    fn to_d2d1_brush(
        &mut self,
        render_target: &ID2D1HwndRenderTarget,
        window_rect: &RECT,
        brush_properties: &D2D1_BRUSH_PROPERTIES,
    ) -> WinResult<()>;
}

pub trait GlobalColorImpl {
    fn to_color(&self, is_active: Option<bool>) -> Result<Color>;
}

impl GlobalColorImpl for GlobalColor {
    fn to_color(&self, is_active: Option<bool>) -> Result<Color> {
        match self {
            GlobalColor::String(s) => parse_color(s.as_str(), is_active),
            GlobalColor::Mapping(gradient_def) => {
                parse_color_mapping(gradient_def.clone(), is_active)
            }
        }
    }
}

impl ColorImpl for Color {
    fn from_global_color(global_color: &GlobalColor, is_active: Option<bool>) -> Result<Self> {
        global_color.to_color(is_active)
    }

    fn set_opacity(&self, opacity: f32) {
        match self {
            Color::Gradient(gradient) => {
                if let Some(ref id2d1_brush) = gradient.brush {
                    unsafe { id2d1_brush.SetOpacity(opacity) }
                }
            }
            Color::Solid(solid) => {
                if let Some(ref id2d1_brush) = solid.brush {
                    unsafe { id2d1_brush.SetOpacity(opacity) }
                }
            }
        }
    }

    fn get_opacity(&self) -> Option<f32> {
        match self {
            Color::Solid(solid) => solid
                .brush
                .as_ref()
                .map(|id2d1_brush| unsafe { id2d1_brush.GetOpacity() }),
            Color::Gradient(gradient) => gradient
                .brush
                .as_ref()
                .map(|id2d1_brush| unsafe { id2d1_brush.GetOpacity() }),
        }
    }

    fn set_transform(&self, transform: &Matrix3x2) {
        match self {
            Color::Solid(solid) => {
                if let Some(ref id2d1_brush) = solid.brush {
                    unsafe {
                        id2d1_brush.SetTransform(transform);
                    }
                }
            }
            Color::Gradient(gradient) => {
                if let Some(ref id2d1_brush) = gradient.brush {
                    unsafe {
                        id2d1_brush.SetTransform(transform);
                    }
                }
            }
        }
    }

    fn get_brush(&self) -> Option<&ID2D1Brush> {
        match self {
            Color::Solid(solid) => solid.brush.as_ref().map(|id2d1_brush| id2d1_brush.into()),
            Color::Gradient(gradient) => gradient
                .brush
                .as_ref()
                .map(|id2d1_brush| id2d1_brush.into()),
        }
    }

    fn to_d2d1_brush(
        &mut self,
        render_target: &ID2D1HwndRenderTarget,
        window_rect: &RECT,
        brush_properties: &D2D1_BRUSH_PROPERTIES,
    ) -> WinResult<()> {
        match self {
            Color::Solid(solid) => unsafe {
                let id2d1_brush =
                    render_target.CreateSolidColorBrush(&solid.color, Some(brush_properties))?;

                id2d1_brush.SetOpacity(0.0);

                solid.brush = Some(id2d1_brush);

                Ok(())
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

                let gradient_stop_collection = render_target.CreateGradientStopCollection(
                    &gradient.gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )?;

                let id2d1_brush = render_target.CreateLinearGradientBrush(
                    &gradient_properties,
                    Some(brush_properties),
                    &gradient_stop_collection,
                )?;

                id2d1_brush.SetOpacity(0.0);
                gradient.brush = Some(id2d1_brush);

                Ok(())
            },
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Solid(Solid {
            color: D2D1_COLOR_F::default(),
            brush: None,
        })
    }
}
