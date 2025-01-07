use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush;

/// Represents a solid color with a specific opacity.
///
/// The `Solid` struct defines a color with an associated opacity.
/// It stores the color as a `D2D1_COLOR_F` struct, which represents the RGBA color values,
/// and the opacity as a `f32` value ranging from 0.0 (fully transparent) to 1.0 (fully opaque).
///
/// # Fields
/// - `color`: A `D2D1_COLOR_F` struct that represents the color in RGBA format, with values for red, green, blue, and alpha (opacity) in the range [0.0, 1.0].
/// - `brush`: An optional `ID2D1SolidColorBrush` that represents the color as a brush, used for rendering the solid color. It may be `None` if not initialized.
///
/// # Example
/// ```rust
/// use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
/// let solid_color = Solid {
///     color: D2D1_COLOR_F { r: 0.5, g: 0.0, b: 0.0, a: 1.0 },
///     brush: None,  // or Some(brush_instance) if a brush is initialized
/// };
/// ```
/// This creates a red color with full opacity and no associated brush.
#[derive(Debug, Clone, PartialEq)]
pub struct Solid {
    pub color: D2D1_COLOR_F,
    pub brush: Option<ID2D1SolidColorBrush>,
}
