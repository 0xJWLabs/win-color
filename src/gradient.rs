use serde::Deserialize;
use std::f32::consts::PI;

use windows::Win32::{
    Foundation::RECT,
    Graphics::Direct2D::{
        Common::{D2D1_GRADIENT_STOP, D2D_POINT_2F},
        ID2D1LinearGradientBrush,
    },
};

use crate::error::WinColorError;

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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
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

/// Defines the coordinates for the start and end points of a gradient.
#[derive(Debug, Clone, Deserialize)]
pub struct GradientCoordinates {
    /// The [x, y] coordinates for the start point of the gradient.
    pub start: [f32; 2],
    /// The [x, y] coordinates for the end point of the gradient.
    pub end: [f32; 2],
}

/// Implements the `TryFrom` trait to convert a string into a `GradientCoordinates` object.
/// The string can represent an angle (e.g., "45deg") or a direction (e.g., "to right").
impl TryFrom<&str> for GradientCoordinates {
    type Error = WinColorError;

    /// Tries to convert a string into a `GradientCoordinates` struct.
    ///
    /// # Parameters
    /// - `color`: A string representing the gradient direction or angle.
    ///
    /// # Returns
    /// A `Result` that is `Ok(GradientCoordinates)` on success or `Err(WinColorError)` on failure.
    fn try_from(color: &str) -> Result<Self, Self::Error> {
        parse_coordinates(color)
    }
}

/// A simple struct representing a line equation (`y = mx + b`).
#[derive(Debug)]
struct Line {
    /// The slope of the line.
    m: f32,
    /// The y-intercept of the line
    b: f32,
}

impl Line {
    /// Calculates the y-value for a given x-value using the line equation `y = mx + b`.
    ///
    /// # Parameters
    /// - `x`: The x-value to plug into the equation.
    ///
    /// # Returns
    /// The y-value corresponding to the given x-value.
    pub fn plug_in_x(&self, x: f32) -> f32 {
        self.m * x + self.b
    }
}

/// Calculates the start and end points of a gradient based on a line equation.
///
/// # Parameters
/// - `line`: A reference to a `Line` struct representing the line equation.
/// - `x`: The x-value to calculate the corresponding y-value for.
///
/// # Returns
/// A 2-element array `[f32; 2]` representing the calculated x and y coordinates.
fn calculate_point(line: &Line, x: f32) -> [f32; 2] {
    match line.plug_in_x(x) {
        0.0..=1.0 => [x, line.plug_in_x(x)],
        1.0.. => [(1.0 - line.b) / line.m, 1.0],
        _ => [-line.b / line.m, 0.0],
    }
}

/// Parses a string representation of gradient coordinates, either as an angle or as a direction.
///
/// # Parameters
/// - `coordinates`: A string representing either an angle or a named direction (e.g., "to right").
///
/// # Returns
/// A `Result` that is `Ok(GradientCoordinates)` on success or `Err(WinColorError)` on failure.
fn parse_coordinates(coordinates: &str) -> Result<GradientCoordinates, WinColorError> {
    let angle = parse_angle(coordinates);

    match angle {
        Some(angle) => {
            let rad = -angle * PI / 180.0;

            let m = match angle.abs() % 360.0 {
                90.0 | 270.0 => angle.signum() * f32::MAX,
                _ => rad.sin() / rad.cos(),
            };

            let b = -m * 0.5 + 0.5;

            let line = Line { m, b };

            let (x_s, x_e) = match angle.abs() % 360.0 {
                0.0..90.0 => (0.0, 1.0),
                90.0..270.0 => (1.0, 0.0),
                270.0..360.0 => (0.0, 1.0),
                _ => (0.0, 1.0),
            };

            let start = calculate_point(&line, x_s);
            let end = calculate_point(&line, x_e);

            // Adjusting calculations based on the origin being (0.5, 0.5)
            Ok(GradientCoordinates { start, end })
        }
        None => match coordinates {
            "to right" => Ok(GradientCoordinates {
                start: [0.0, 0.5],
                end: [1.0, 0.5],
            }),
            "to left" => Ok(GradientCoordinates {
                start: [1.0, 0.5],
                end: [0.0, 0.5],
            }),
            "to top" => Ok(GradientCoordinates {
                start: [0.5, 1.0],
                end: [0.5, 0.0],
            }),
            "to bottom" => Ok(GradientCoordinates {
                start: [0.5, 0.0],
                end: [0.5, 1.0],
            }),
            "to top right" => Ok(GradientCoordinates {
                start: [0.0, 1.0],
                end: [1.0, 0.0],
            }),
            "to top left" => Ok(GradientCoordinates {
                start: [1.0, 1.0],
                end: [0.0, 0.0],
            }),
            "to bottom right" => Ok(GradientCoordinates {
                start: [0.0, 0.0],
                end: [1.0, 1.0],
            }),
            "to bottom left" => Ok(GradientCoordinates {
                start: [1.0, 0.0],
                end: [0.0, 1.0],
            }),
            _ => Err(WinColorError::InvalidGradientCoordinates(
                coordinates.to_string(),
            )),
        },
    }
}

/// Parses a string representing an angle and converts it to radians or degrees.
///
/// The angle can be in various units such as "deg", "grad", "rad", or "turn".
/// The function attempts to parse the angle and convert it into a float value representing the angle in radians.
///
/// If no valid angle or unit is found, `None` is returned.
///
/// # Parameters
/// - `s`: A string representing an angle. This string can have a suffix indicating the unit of measurement, such as "deg", "grad", "rad", or "turn".
///
/// # Returns
/// Returns an `Option<f32>`:
/// - `Some(f32)` if the string is a valid angle with a recognized unit or as a plain number.
/// - `None` if the string cannot be parsed as a valid angle.
fn parse_angle(s: &str) -> Option<f32> {
    s.strip_suffix("deg")
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            s.strip_suffix("grad")
                .and_then(|s| s.parse().ok())
                .map(|t: f32| t * 360.0 / 400.0)
        })
        .or_else(|| {
            s.strip_suffix("rad")
                .and_then(|s| s.parse().ok())
                .map(|t: f32| t.to_degrees())
        })
        .or_else(|| {
            s.strip_suffix("turn")
                .and_then(|s| s.parse().ok())
                .map(|t: f32| t * 360.0)
        })
        .or_else(|| s.parse().ok())
}

pub fn is_valid_direction(direction: &str) -> bool {
    matches!(
        direction,
        "to right"
            | "to left"
            | "to top"
            | "to bottom"
            | "to top right"
            | "to top left"
            | "to bottom right"
            | "to bottom left"
    ) || is_valid_angle(direction)
}

fn is_valid_angle(direction: &str) -> bool {
    const VALID_SUFFIXES: [&str; 4] = ["deg", "grad", "rad", "turn"];

    VALID_SUFFIXES.iter().any(|&suffix| {
        direction
            .strip_suffix(suffix) // Remove the suffix
            .and_then(|num| num.parse::<f32>().ok()) // Parse the numeric part
            .is_some()
    })
}
