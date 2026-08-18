use image::Rgb;
use noise::{MultiFractal, NoiseFn};
use rand::prelude::*;
use tracing::{info, trace};

use crate::hex::Grid;
use crate::rng::{Dice, RngMaster};

#[derive(Debug, Clone, Copy)]
struct RgbLerpPoints {
    d: f64,
    r: u8,
    g: u8,
    b: u8,
}

#[rustfmt::skip]
const BROWN_LERP: [RgbLerpPoints; 6] = [
    RgbLerpPoints { d: 0.0, r: 0x00, g: 0x00, b: 0x00 },
    RgbLerpPoints { d: 0.2, r: 0x1a, g: 0x1a, b: 0x1a },
    RgbLerpPoints { d: 0.4, r: 0x4a, g: 0x35, b: 0x20 },
    RgbLerpPoints { d: 0.6, r: 0x8b, g: 0x62, b: 0x39 },
    RgbLerpPoints { d: 0.8, r: 0xc2, g: 0xa2, b: 0x78 },
    RgbLerpPoints { d: 1.0, r: 0xff, g: 0xff, b: 0xff },
];

// expects input in range -1.0 to 1.0, as returned by NoiseFn
// clamp shouldn't be necessary, but just-in-case
#[inline(always)]
fn normalise(i: f64) -> f64 {
    ((i + 1.0) / 2.0).clamp(0.0, 1.0)
}

#[tracing::instrument(level = "trace")]
pub fn brownscale(i: &f64) -> Rgb<u8> {
    let i = normalise(*i);
    let mut lower = BROWN_LERP[0];
    let mut upper = BROWN_LERP[0];
    for p in BROWN_LERP {
        lower = upper;
        upper = p;
        if i <= p.d {
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

#[tracing::instrument(level = "trace")]
pub fn brown_palette(i: &f64) -> Rgb<u8> {
    let i = normalise(*i);
    trace!(normalised_i = i);
    let mut point: RgbLerpPoints = BROWN_LERP[0];
    for p in BROWN_LERP {
        point = p;
        if i <= point.d {
            break;
        }
    }
    trace!(?point);
    Rgb([point.r, point.g, point.b])
}

#[tracing::instrument(level = "debug")]
pub fn generate(rng_master: RngMaster, size: usize) -> Result<Grid<f64>, String> {
    let mut rng = rng_master.for_stage("elevation");
    let noise_seed = rng.next_u32();
    trace!(%noise_seed);
    let scale = rng.d(20);
    let frequency = f64::from(scale) / size as f64;
    info!(%scale, "Randomly-chosen parameter for elevation noise");

    let fbm = noise::Fbm::<noise::PerlinSurflet>::new(noise_seed).set_frequency(frequency);
    trace!(
        origin = fbm.get([0.0, 0.0]),
        _0_5_0_5 = fbm.get([0.5, 0.5]),
        size_size = fbm.get([size as f64, size as f64]),
        "Noise values at"
    );
    let grid = Grid::new_with_world_coords(size, size, |(col, row)| fbm.get([col, row])).unwrap();
    trace!(nans_in_grid = grid.iter().any(|n| n.is_nan()));
    let grid_min = grid.iter().copied().fold(f64::INFINITY, f64::min);
    let grid_max = grid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let grid_mean = grid.iter().sum::<f64>() / grid.len() as f64;
    trace!(grid_min, grid_max, grid_mean);
    Ok(grid)
}
