use serde::Deserialize;

use crate::GradientCoordinates;
use windows::Win32::{
    Foundation::RECT,
    Graphics::Direct2D::{
        Common::{D2D1_GRADIENT_STOP, D2D_POINT_2F},
        ID2D1LinearGradientBrush,
    },
};

#[allow(dead_code)]
pub trait GradientImpl {
    /// Updates the start and end points of the gradient based on the window's dimensions.
    ///
    /// This method calculates the new start and end points of the gradient brush based on the
    /// window's size (given by `window_rect`). The direction of the gradient is scaled from
    /// normalized coordinates (ranging from 0.0 to 1.0) to pixel coordinates based on the window's
    /// width and height.
    ///
    /// # Parameters
    /// - `window_rect`: The dimensions of the window as a `RECT`, used to compute the pixel
    ///   positions for the gradient's start and end points.
    fn update_start_end_points(&self, window_rect: &RECT);
}

/// Represents a gradient with a specific direction, gradient stops, and an optional brush.
///
/// The `Gradient` struct defines a linear gradient that can be applied to render objects with
/// smooth transitions between colors. The gradient's direction and color stops determine how the
/// gradient appears, while the optional brush holds the gradient data for rendering.
///
/// # Fields
/// - `direction`: Specifies the gradient's direction, given by `GradientCoordinates`. It consists of
///   start and end points, each defined as normalized values ranging from 0.0 to 1.0. These values
///   are scaled based on the size of the window to determine the pixel positions of the gradient's
///   start and end points.
/// - `gradient_stops`: A vector of `D2D1_GRADIENT_STOP` values, representing the color stops in the
///   gradient. These stops define the colors that the gradient transitions through.
/// - `brush`: An optional `ID2D1LinearGradientBrush` used to render the gradient. If not initialized,
///   this value is `None`.
///
/// # Example
/// ```rust
/// use windows::Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP;
/// let gradient = Gradient {
///     direction: GradientCoordinates { start: [0.0, 0.0], end: [1.0, 1.0] },
///     gradient_stops: vec![
///         D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 0.0, b: 0.0, a: 1.0 } },
///         D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 1.0, a: 1.0 } },
///     ],
///     brush: None, // Brush will be initialized later
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    /// The direction of the gradient, either as a string or as coordinates.
    pub direction: GradientCoordinates,
    /// A list of gradient stops defining color stops in the gradient.
    pub gradient_stops: Vec<D2D1_GRADIENT_STOP>,

    /// An optional linear gradient brush that can be used for rendering the gradient.
    /// It represents the gradient with a direction and color stops, and may be `None` if not yet initialized.
    pub brush: Option<ID2D1LinearGradientBrush>,
}

impl GradientImpl for Gradient {
    fn update_start_end_points(&self, window_rect: &RECT) {
        let width = (window_rect.right - window_rect.left) as f32;
        let height = (window_rect.bottom - window_rect.top) as f32;

        // The direction/GradientCoordinates only range from 0.0 to 1.0, but we need to
        // convert it into coordinates in terms of pixels
        let start_point = D2D_POINT_2F {
            x: self.direction.start[0] * width,
            y: self.direction.start[1] * height,
        };
        let end_point = D2D_POINT_2F {
            x: self.direction.end[0] * width,
            y: self.direction.end[1] * height,
        };

        if let Some(ref id2d1_brush) = self.brush {
            unsafe {
                id2d1_brush.SetStartPoint(start_point);
                id2d1_brush.SetEndPoint(end_point)
            };
        }
    }
}

/// Enum representing different types of gradient directions.
/// It can either be a string describing the direction (e.g., "to right") or explicit coordinates for the gradient direction.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GradientDirection {
    /// Direction is represented as a string (e.g., "to right").
    Direction(String),
    /// Direction is represented as coordinates (e.g., GradientCoordinates).
    Coordinates(GradientCoordinates),
}

impl From<&str> for GradientDirection {
    fn from(s: &str) -> Self {
        Self::Direction(s.to_string())
    }
}

/// A structure that defines a gradient mapping, which contains a list of color stops and a direction.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ColorMapping {
    /// A list of colors in the gradient, represented as hexadecimal color strings.
    pub colors: Vec<String>,
    /// The direction of the gradient, represented as a `GradientDirection`.
    pub direction: GradientDirection,
}

pub trait ColorMappingImpl {
    fn new(colors: &[&str], direction: GradientDirection) -> Self;
}

impl ColorMappingImpl for ColorMapping {
    fn new(colors: &[&str], direction: GradientDirection) -> Self {
        Self {
            colors: colors.iter().map(|&s| s.to_string()).collect(),
            direction,
        }
    }
}
