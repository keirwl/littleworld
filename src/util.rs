use image::Rgb;
use tracing::trace;

pub const SQRT_3: f64 = 1.7320508075688772;

#[derive(Debug, Clone, Copy)]
pub struct RgbLerpPoints {
    pub d: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[tracing::instrument(level = "trace")]
pub fn lerp(points: &[RgbLerpPoints], i: &f64) -> Rgb<u8> {
    let mut lower = points[0];
    if *i <= 0.0 {
        return Rgb([lower.r, lower.g, lower.b]);
    }
    let mut upper = points[0];
    for p in points {
        lower = upper;
        upper = *p;
        if *i <= p.d {
            break;
        }
    }
    trace!(?lower, ?upper);
    let f = (i - lower.d) / (upper.d - lower.d);
    trace!(i, ?lower.d, ?upper.d, f);
    Rgb([
        (lower.r as f64 + (upper.r as f64 - lower.r as f64) * f).clamp(0.0, 255.0) as u8,
        (lower.g as f64 + (upper.g as f64 - lower.g as f64) * f).clamp(0.0, 255.0) as u8,
        (lower.b as f64 + (upper.b as f64 - lower.b as f64) * f).clamp(0.0, 255.0) as u8,
    ])
}
