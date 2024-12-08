use core::f32;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

pub fn strip_string(input: String, prefixes: &[&str], suffix: char) -> String {
    let mut result = input;

    // Remove matching prefix (if any)
    for &prefix in prefixes {
        if let Some(stripped) = result.strip_prefix(prefix) {
            result = stripped.to_string();
            break; // Only remove the first matching prefix
        }
    }

    // Remove suffix (if it exists)
    result.strip_suffix(suffix).unwrap_or(&result).to_string()
}

#[derive(Debug, Clone)]
struct Hsla {
    h: f32,
    s: f32,
    l: f32,
    a: f32,
}

fn d2d1_to_hsla(color: D2D1_COLOR_F) -> Hsla {
    let r = color.r;
    let g = color.g;
    let b = color.b;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let mut h = 0.0;
    let mut s = 0.0;
    let mut l = (max + min) / 2.0;

    if delta != 0.0 {
        if max == r {
            h = (g - b) / delta;
        } else if max == g {
            h = (b - r) / delta + 2.0;
        } else {
            h = (r - g) / delta + 4.0;
        }

        s = if l == 0.0 || l == 1.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }
    }

    s *= 100.0;
    l *= 100.0;

    Hsla {
        h,
        s,
        l,
        a: color.a,
    }
}

fn hsla_to_d2d1(hsla: Hsla) -> D2D1_COLOR_F {
    let s = hsla.s / 100.0;
    let l = hsla.l / 100.0;
    let h = hsla.h;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    D2D1_COLOR_F {
        r: (r + m).clamp(0.0, 1.0),
        g: (g + m).clamp(0.0, 1.0),
        b: (b + m).clamp(0.0, 1.0),
        a: hsla.a,
    }
}

pub fn darken(color: D2D1_COLOR_F, percentage: f32) -> D2D1_COLOR_F {
    let mut hsla = d2d1_to_hsla(color);
    hsla.l -= hsla.l * percentage / 100.0;
    hsla_to_d2d1(hsla)
}

pub fn lighten(color: D2D1_COLOR_F, percentage: f32) -> D2D1_COLOR_F {
    let mut hsla = d2d1_to_hsla(color);
    hsla.l += hsla.l * percentage / 100.0;
    hsla_to_d2d1(hsla)
}
