use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// Represents a solid color with a specific opacity.
///
/// The `Solid` struct is used to define a color with an associated opacity.
/// It stores the color as a `D2D1_COLOR_F`, which represents the RGBA color values, and the opacity as a `f32` value.
///
/// # Fields
/// - `color`: A `D2D1_COLOR_F` struct that represents the color in the form of RGBA values. The values for red, green, blue, and alpha (opacity) range from 0.0 to 1.0.
/// - `opacity`: A `f32` value representing the opacity of the color. A value of `1.0` is fully opaque, and `0.0` is fully transparent.
///
/// # Example
/// ```rust
/// use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
/// let solid_color = Solid {
///     color: D2D1_COLOR_F { r: 0.5, g: 0.0, b: 0.0, a: 1.0 },
///     opacity: 0.8,
/// };
/// ```
/// This creates a red color with 80% opacity.
#[derive(Debug, Clone, PartialEq)]
pub struct Solid {
    pub color: D2D1_COLOR_F,
    pub opacity: f32,
}
