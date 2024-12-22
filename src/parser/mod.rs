//! This module handles named colors and related utilities.

mod named_colors;

pub use named_colors::COLOR_REGEX;
use named_colors::DARKEN_LIGHTEN_REGEX;
pub use named_colors::NAMED_COLORS;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::FALSE;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP;
use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;

use crate::error::Error;
use crate::error::ErrorKind;
use crate::error::Result;
use crate::gradient::is_valid_direction;
use crate::utils::darken;
use crate::utils::lighten;
use crate::utils::strip_string;
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
pub fn parse_color_mapping(s: ColorMapping, is_active: Option<bool>) -> Result<Color> {
    match s.colors.len() {
        0 => Ok(Color::Solid(Solid {
            color: D2D1_COLOR_F::default(),
            brush: None,
        })),
        1 => Ok(Color::Solid(Solid {
            color: parse(s.colors[0].as_str(), is_active)?,
            brush: None,
        })),
        _ => {
            let num_colors = s.colors.len();
            let step = 1.0 / (num_colors - 1) as f32;
            let gradient_stops: Vec<D2D1_GRADIENT_STOP> = s
                .colors
                .iter()
                .enumerate()
                .filter_map(|(i, hex)| {
                    parse(hex, is_active).ok().map(|color| D2D1_GRADIENT_STOP {
                        position: i as f32 * step,
                        color,
                    })
                })
                .collect(); // Collect the successful results into a Vec

            let direction = match s.direction {
                GradientDirection::Direction(direction) => {
                    GradientCoordinates::try_from(direction.as_str())
                }
                GradientDirection::Coordinates(direction) => Ok(direction),
            }?;

            Ok(Color::Gradient(Gradient {
                gradient_stops,
                direction,
                brush: None,
            }))
        }
    }
}

/// Parses a color string and returns a `Color` enum, or an error if the string is invalid.
///
/// This function supports various formats such as RGB, RGBA, hex, gradient, and special values like `accent`.
///
/// # Parameters
/// - `s`: The color string to parse.
/// - `is_active`: Optional boolean that determines the behavior of color parsing for active states.
///
/// # Returns
/// - `Ok(Color)` on successful parsing.
/// - `Err(WinColorError)` if the color string is invalid.
///
/// # Examples
///
/// ```rust
/// let color = parse_color("#89b4fa", Some(false))?;
/// ```
pub fn parse_color(s: &str, is_active: Option<bool>) -> Result<Color> {
    if s.starts_with("gradient(") && s.ends_with(")") {
        return parse_color(
            strip_string(s.to_string(), &["gradient("], ')').as_str(),
            is_active,
        );
    }

    let color_re = &COLOR_REGEX;

    let color_matches: Vec<&str> = color_re
        .captures_iter(s)
        .filter_map(|cap| cap.get(0).map(|m| m.as_str()))
        .collect();

    if color_matches.len() == 1 {
        let color = parse(color_matches[0], is_active)?;

        return Ok(Color::Solid(Solid { color, brush: None }));
    }

    let remaining_input = s
        [s.rfind(color_matches.last().unwrap()).unwrap() + color_matches.last().unwrap().len()..]
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
        .filter_map(|&color| parse(color, is_active).ok()) // Only keep Ok values
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

    Ok(Color::Gradient(Gradient {
        gradient_stops,
        direction,
        brush: None,
    }))
}

fn parse(s: &str, is_active: Option<bool>) -> Result<D2D1_COLOR_F> {
    if s == "accent" {
        let mut pcr_colorization: u32 = 0;
        let mut pf_opaqueblend: BOOL = FALSE;

        if unsafe { DwmGetColorizationColor(&mut pcr_colorization, &mut pf_opaqueblend) }.is_err() {
            return Err(Error::new(
                ErrorKind::InvalidAccent,
                "accent color not found",
            ));
        }

        let r = ((pcr_colorization & 0x00FF0000) >> 16) as f32 / 255.0;
        let g = ((pcr_colorization & 0x0000FF00) >> 8) as f32 / 255.0;
        let b = (pcr_colorization & 0x000000FF) as f32 / 255.0;
        let avg = (r + g + b) / 3.0;

        return match is_active {
            Some(true) => Ok(D2D1_COLOR_F { r, g, b, a: 1.0 }),
            _ => Ok(D2D1_COLOR_F {
                r: avg / 1.5 + r / 10.0,
                g: avg / 1.5 + g / 10.0,
                b: avg / 1.5 + b / 10.0,
                a: 1.0,
            }),
        };
    }

    if let Some(color) = NAMED_COLORS.get(s) {
        return Ok(*color);
    }

    if let Some(s) = s.strip_prefix("#") {
        return parse_hex(s);
    }

    if s.starts_with("rgb(") || s.starts_with("rgba(") {
        let rgba = strip_string(s.to_string(), &["rgb(", "rgba("], ')');
        let params: Vec<&str> = rgba.split(',').map(|s| s.trim()).collect();

        if params.len() != 3 && params.len() != 4 {
            return Err(Error::new(ErrorKind::InvalidRgb, s));
        }

        let r = parse_percent_or_255(params[0]);
        let g = parse_percent_or_255(params[1]);
        let b = parse_percent_or_255(params[2]);

        let a = if params.len() == 4 {
            parse_percent_or_float(params[3])
        } else {
            Some((1.0, true))
        };

        if let (Some((r, r_fmt)), Some((g, g_fmt)), Some((b, b_fmt)), Some((a, _))) = (r, g, b, a) {
            if r_fmt == g_fmt && g_fmt == b_fmt {
                return Ok(D2D1_COLOR_F {
                    r: r.clamp(0.0, 1.0),
                    g: g.clamp(0.0, 1.0),
                    b: b.clamp(0.0, 1.0),
                    a: a.clamp(0.0, 1.0),
                });
            }
        }

        return Err(Error::new(ErrorKind::InvalidRgb, s));
    } else if s.starts_with("darken(") || s.starts_with("lighten(") {
        let darken_lighten_re = &DARKEN_LIGHTEN_REGEX;

        if let Some(caps) = darken_lighten_re.captures(s) {
            if caps.len() != 4 {
                if s.starts_with("darken(") {
                    return Err(Error::new(ErrorKind::InvalidDarken, s));
                }
                return Err(Error::new(ErrorKind::InvalidLighten, s));
            }
            let dark_or_lighten = &caps[1];
            let color_str = &caps[2];
            let percentage = &caps[3].parse::<f32>().unwrap_or(10.0);

            let color = parse(color_str, is_active)?;
            let color_res = match dark_or_lighten {
                "darken" => darken(color, *percentage),
                "lighten" => lighten(color, *percentage),
                _ => color,
            };

            return Ok(color_res);
        }

        if s.starts_with("darken(") {
            return Err(Error::new(ErrorKind::InvalidDarken, s));
        }
        return Err(Error::new(ErrorKind::InvalidLighten, s));
    }

    Ok(D2D1_COLOR_F::default())
}

fn parse_hex(s: &str) -> Result<D2D1_COLOR_F> {
    if !s.is_ascii() {
        return Err(Error::new(ErrorKind::InvalidHex, s));
    }

    let n = s.len();

    fn parse_single_digit(digit: &str) -> Result<f32> {
        u8::from_str_radix(digit, 16)
            .map(|n| ((n << 4) | n) as f32 / 255.0)
            .map_err(|_| Error::new(ErrorKind::InvalidHex, digit))
    }

    if n == 3 || n == 4 {
        let r = parse_single_digit(&s[0..1])?;
        let g = parse_single_digit(&s[1..2])?;
        let b = parse_single_digit(&s[2..3])?;

        let a = if n == 4 {
            parse_single_digit(&s[3..4])?
        } else {
            1.0
        };

        Ok(D2D1_COLOR_F { r, g, b, a })
    } else if n == 6 || n == 8 {
        let r = u8::from_str_radix(&s[0..2], 16)
            .map(|n| n as f32 / 255.0)
            .map_err(|_| Error::new(ErrorKind::InvalidHex, s))?;
        let g = u8::from_str_radix(&s[2..4], 16)
            .map(|n| n as f32 / 255.0)
            .map_err(|_| Error::new(ErrorKind::InvalidHex, s))?;
        let b = u8::from_str_radix(&s[4..6], 16)
            .map(|n| n as f32 / 255.0)
            .map_err(|_| Error::new(ErrorKind::InvalidHex, s))?;

        let a = if n == 8 {
            u8::from_str_radix(&s[6..8], 16)
                .map(|n| n as f32 / 255.0)
                .map_err(|_| Error::new(ErrorKind::InvalidHex, s))?
        } else {
            1.0
        };

        Ok(D2D1_COLOR_F { r, g, b, a })
    } else {
        Err(Error::new(ErrorKind::InvalidHex, s))
    }
}

fn parse_percent_or_float(s: &str) -> Option<(f32, bool)> {
    s.strip_suffix('%')
        .and_then(|s| s.parse().ok().map(|t: f32| (t / 100.0, true)))
        .or_else(|| s.parse().ok().map(|t| (t, false)))
}

fn parse_percent_or_255(s: &str) -> Option<(f32, bool)> {
    s.strip_suffix('%')
        .and_then(|s| s.parse().ok().map(|t: f32| (t / 100.0, true)))
        .or_else(|| s.parse().ok().map(|t: f32| (t / 255.0, false)))
}
