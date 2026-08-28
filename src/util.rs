use image::Rgb;

pub const SQRT_3: f64 = 1.7320508075688772;

#[derive(Debug, Clone, Copy)]
pub struct RgbLerpPoints {
    pub d: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// #[tracing::instrument(level = "trace")]
#[inline]
pub fn lerp(points: &[RgbLerpPoints], i: &f64) -> Rgb<u8> {
    let mut lower = points[0];
    let mut upper = points[1];
    for w in points.windows(2) {
        (lower, upper) = (w[0], w[1]);
        if *i <= upper.d {
            break;
        }
    }
    let f = (i - lower.d) / (upper.d - lower.d);
    Rgb([
        (lower.r as f64 + (upper.r as f64 - lower.r as f64) * f).clamp(0.0, 255.0) as u8,
        (lower.g as f64 + (upper.g as f64 - lower.g as f64) * f).clamp(0.0, 255.0) as u8,
        (lower.b as f64 + (upper.b as f64 - lower.b as f64) * f).clamp(0.0, 255.0) as u8,
    ])
}

#[inline]
pub fn smootherstep(x: f64, min: f64, max: f64) -> f64 {
    let x = ((x - min) / (max - min)).clamp(0.0, 1.0);
    x * x * x * (x * (6.0 * x - 15.0) + 10.0)
}

// expects input in range -1.0 to 1.0, as returned by NoiseFn
#[inline]
pub fn normalise(i: f64) -> f64 {
    (i + 1.0) / 2.0
}
