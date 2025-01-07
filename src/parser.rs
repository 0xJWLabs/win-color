//! This module handles named colors and related utilities for parsing and managing colors.
//! It supports solid colors, gradients, and their mapping to Direct2D structures.

use colorparser_css::Color as CssColor;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP;

use crate::error::Error;
use crate::error::ErrorKind;
use crate::error::Result;
use crate::Color;
use crate::ColorMapping;
use crate::Gradient;
use crate::GradientCoordinates;
use crate::GradientDirection;
use crate::Solid;

/// Parses a `ColorMapping` into a `Color`.
///
/// # Arguments
///
/// - `s`: A `ColorMapping` containing the color definitions to parse.
/// - `is_active`: An optional flag indicating whether the color mapping is active.
///
/// # Returns
///
/// - `Ok(Color)`: A parsed `Color` object, which can be a solid color or a gradient.
/// - `Err(WinColorError)`: An error if the parsing fails.
///
/// # Examples
///
/// ```rust
/// let mapping = ColorMapping {
///     colors: vec!["#FF0000".to_string()],
///     direction: GradientCoordinates::String("90deg".to_string())
/// };
/// let color = parse_color_mapping(mapping, Some(false))?;
/// ```
pub fn parse_color_mapping(s: ColorMapping) -> Result<Color> {
    match s.colors.len() {
        0 => Ok(Color::Solid(Solid {
            color: D2D1_COLOR_F::default(),
            brush: None,
        })),
        1 => {
            let result = parse_color_string(&s.colors[0])?;
            Ok(result)
        }
        _ => {
            let gradient_stops = generate_gradient_stops(&s.colors)?;

            if gradient_stops.is_empty() {
                return Err(Error::new(ErrorKind::InvalidData, "No valid colors found"));
            }

            let direction = parse_gradient_direction(&s.direction)?;

            Ok(Color::Gradient(Gradient {
                gradient_stops,
                direction,
                brush: None,
            }))
        }
    }
}

/// Generates gradient stops from a list of color strings.
///
/// # Arguments
///
/// - `colors`: A slice of strings representing color values in CSS-compatible format.
///
/// # Returns
///
/// - `Ok(Vec<D2D1_GRADIENT_STOP>)`: A vector of gradient stops for use with Direct2D.
/// - `Err(Error)`: An error if color parsing fails.
///
/// # Examples
///
/// ```rust
/// let stops = generate_gradient_stops(&vec!["#FF0000".to_string(), "#00FF00".to_string()])?;
/// ```
fn generate_gradient_stops(colors: &[String]) -> Result<Vec<D2D1_GRADIENT_STOP>> {
    let num_colors = colors.len();
    let step = 1.0 / (num_colors - 1) as f32;

    let stops: Vec<D2D1_GRADIENT_STOP> = colors
        .iter()
        .enumerate()
        .filter_map(|(i, hex)| match parse_color_string(hex).ok()? {
            Color::Solid(solid) => Some(D2D1_GRADIENT_STOP {
                position: i as f32 * step,
                color: solid.color,
            }),
            _ => None, // Skip invalid colors
        })
        .collect();

    Ok(stops)
}

/// Parses a gradient direction into `GradientCoordinates`.
///
/// # Arguments
///
/// - `direction`: A `GradientDirection` enum specifying the gradient's direction or coordinates.
///
/// # Returns
///
/// - `Ok(GradientCoordinates)`: A valid gradient coordinate mapping.
/// - `Err(Error)`: An error if the direction is invalid.
///
/// # Examples
///
/// ```rust
/// let direction = GradientDirection::Direction("90deg".to_string());
/// let coordinates = parse_gradient_direction(&direction)?;
/// ```
fn parse_gradient_direction(direction: &GradientDirection) -> Result<GradientCoordinates> {
    match direction {
        GradientDirection::Direction(dir) => {
            GradientCoordinates::try_from(dir.as_str()).map_err(|e| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid gradient direction: {}", e),
                )
            })
        }
        GradientDirection::Coordinates(coords) => Ok(coords.clone()),
    }
}

/// Parses a CSS color string into a `Color`.
///
/// This function supports solid colors and gradients in CSS-compatible formats.
///
/// # Arguments
///
/// - `s`: A string containing the CSS color definition.
///
/// # Returns
///
/// - `Ok(Color)`: A parsed `Color` object.
/// - `Err(Error)`: An error if the input is invalid or unsupported.
///
/// # Examples
///
/// ```rust
/// let color = parse_color_string("#FF0000")?;
/// ```
pub fn parse_color_string(s: &str) -> Result<Color> {
    let css_color = CssColor::from_html(s).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("CSS parsing failed: {}", e),
        )
    })?;

    parse_solid_color(&css_color)
        .or_else(|_| parse_gradient(&css_color))
        .map_err(|_| {
            Error::new(
                ErrorKind::InvalidInput,
                "Input does not represent a valid solid color or gradient",
            )
        })
}

/// Parses a `CssColor` into a solid `Color`.
///
/// # Arguments
///
/// - `css_color`: A `CssColor` object representing a solid color.
///
/// # Returns
///
/// - `Ok(Color::Solid)`: A `Solid` color object.
/// - `Err(Error)`: An error if the input is not a solid color.
///
/// # Examples
///
/// ```rust
/// let color = parse_solid_color(&CssColor::from_html("#FF0000")?)?;
/// ```
fn parse_solid_color(css_color: &CssColor) -> Result<Color> {
    let solid = css_color
        .to_solid()
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Not a solid color"))?;
    let normalized_rgba = solid.to_normalized_rgba();
    let color = D2D1_COLOR_F {
        r: normalized_rgba.r,
        g: normalized_rgba.g,
        b: normalized_rgba.b,
        a: normalized_rgba.a,
    };
    Ok(Color::Solid(Solid { color, brush: None }))
}

/// Parses a `CssColor` into a gradient `Color`.
///
/// # Arguments
///
/// - `css_color`: A `CssColor` object representing a gradient.
///
/// # Returns
///
/// - `Ok(Color::Gradient)`: A `Gradient` color object.
/// - `Err(Error)`: An error if the input is not a gradient.
///
/// # Examples
///
/// ```rust
/// let color = parse_gradient(&CssColor::from_html("linear-gradient(to right, #FF0000, #00FF00)")?)?;
/// ```
fn parse_gradient(css_color: &CssColor) -> Result<Color> {
    let gradient = css_color
        .to_gradient()
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Not a gradient"))?;
    let num_colors = gradient.colors.len();
    let step = 1.0 / (num_colors - 1) as f32;

    let gradient_stops: Vec<D2D1_GRADIENT_STOP> = gradient
        .colors
        .into_iter()
        .enumerate()
        .map(|(i, solid)| {
            let normalized_rgba = solid.to_normalized_rgba();
            let color = D2D1_COLOR_F {
                r: normalized_rgba.r,
                g: normalized_rgba.g,
                b: normalized_rgba.b,
                a: normalized_rgba.a,
            };
            D2D1_GRADIENT_STOP {
                position: i as f32 * step,
                color,
            }
        })
        .collect();

    let direction = GradientCoordinates {
        start: gradient.direction.start,
        end: gradient.direction.end,
    };

    Ok(Color::Gradient(Gradient {
        direction,
        gradient_stops,
        brush: None,
    }))
}
