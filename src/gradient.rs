use serde::Deserialize;
use std::f32::consts::PI;

use windows::Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP;

use crate::error::WinColorError;

/// Represents a gradient with a specific direction, gradient stops, and opacity.
#[derive(Debug, Clone)]
pub struct Gradient {
    /// The direction of the gradient, either as a string or as coordinates.
    pub direction: GradientCoordinates,
    /// A list of gradient stops defining color stops in the gradient.
    pub gradient_stops: Vec<D2D1_GRADIENT_STOP>,
    /// The opacity of the gradient, represented as a floating-point number between 0.0 and 1.0.
    pub opacity: f32,
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

/// A structure that defines a gradient mapping, which contains a list of color stops and a direction.
#[derive(Debug, Clone, Deserialize)]
pub struct GradientMapping {
    /// A list of colors in the gradient, represented as hexadecimal color strings.
    pub colors: Vec<String>,
    /// The direction of the gradient, represented as a `GradientDirection`.
    pub direction: GradientDirection,
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
