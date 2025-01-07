//! This module handles named colors and related utilities.

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
            let num_colors = s.colors.len();
            let step = 1.0 / (num_colors - 1) as f32;

            let gradient_stops: Vec<D2D1_GRADIENT_STOP> = s
                .colors
                .iter()
                .enumerate()
                .filter_map(|(i, hex)| {
                    match parse_color_string(hex).ok()? {
                        Color::Solid(solid) => Some(D2D1_GRADIENT_STOP {
                            position: i as f32 * step,
                            color: solid.color,
                        }),
                        _ => None, // Skip gradients or unsupported colors
                    }
                })
                .collect();

            if gradient_stops.is_empty() {
                return Err(Error::new(ErrorKind::InvalidData, "No valid colors found"));
            }

            let direction = match &s.direction {
                GradientDirection::Direction(dir) => {
                    // Ensure proper error handling and mapping
                    GradientCoordinates::try_from(dir.as_str())
                        .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?
                }
                GradientDirection::Coordinates(coords) => coords.clone(),
            };

            Ok(Color::Gradient(Gradient {
                gradient_stops,
                direction,
                brush: None,
            }))
        }
    }
}

pub fn parse_color_string(s: &str) -> Result<Color> {
    let css_color = CssColor::from_html(s).map_err(|e| {
        Error::new(
            ErrorKind::InvalidUnknown,
            format!("CSS parsing failed: {}", e),
        )
    })?;

    if let Ok(solid) = css_color.to_solid() {
        let normalized_rgba = solid.to_normalized_rgba();
        let color = D2D1_COLOR_F {
            r: normalized_rgba.r,
            g: normalized_rgba.g,
            b: normalized_rgba.b,
            a: normalized_rgba.a,
        };

        return Ok(Color::Solid(Solid { color, brush: None }));
    } else if let Ok(gradient) = css_color.to_gradient() {
        let direction = gradient.direction;
        let colors = gradient.colors;

        let num_colors = colors.len();

        let step = 1.0 / (num_colors - 1) as f32;
        let gradient_stops: Vec<D2D1_GRADIENT_STOP> = colors
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
            start: direction.start,
            end: direction.end,
        };

        return Ok(Color::Gradient(Gradient {
            direction,
            gradient_stops,
            brush: None,
        }));
    }

    Err(Error::new(
        ErrorKind::InvalidUnknown,
        "Input does not represent a valid solid color or gradient",
    ))
}
