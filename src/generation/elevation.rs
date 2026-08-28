use std::f64::consts;

use image::Rgb;
use noise::{MultiFractal, NoiseFn};
use rand::prelude::*;
use tracing::{info, trace};

use crate::generation::elevation::LandmassType::Island;
use crate::hex::Grid;
use crate::rng::{Dice, RngMaster};
use crate::util::{RgbLerpPoints, lerp, normalise, smootherstep};

#[rustfmt::skip]
const WATER_LERP: [RgbLerpPoints; 4] = [
    RgbLerpPoints { d: -1.00, r: 0x5e, g: 0x8f, b: 0xbf }, // deep
    RgbLerpPoints { d: -0.40, r: 0x8a, g: 0xba, b: 0xe3 }, // mid
    RgbLerpPoints { d: -0.10, r: 0xac, g: 0xdb, b: 0xfb }, // shelf
    RgbLerpPoints { d:  0.00, r: 0xd8, g: 0xf2, b: 0xfe }, // shore
];

#[rustfmt::skip]
const LAND_LERP: [RgbLerpPoints; 8] = [
    RgbLerpPoints { d: 0.000, r: 0x94, g: 0xbf, b: 0x8b }, // 0 m, green
    RgbLerpPoints { d: 0.050, r: 0xbd, g: 0xcc, b: 0x96 }, // 100 m
    RgbLerpPoints { d: 0.125, r: 0xef, g: 0xeb, b: 0xc0 }, // 250 m, cream
    RgbLerpPoints { d: 0.250, r: 0xde, g: 0xd6, b: 0xa3 }, // 500 m
    RgbLerpPoints { d: 0.500, r: 0xca, g: 0xb9, b: 0x82 }, // 1000 m
    RgbLerpPoints { d: 0.750, r: 0xb9, g: 0x98, b: 0x5a }, // 1500 m, tan
    RgbLerpPoints { d: 0.900, r: 0xac, g: 0x9a, b: 0x7c }, // 1800 m, rock
    RgbLerpPoints { d: 1.000, r: 0xf0, g: 0xf0, b: 0xf0 }, // 2000 m
];

pub fn colour_land_sea(i: &f64) -> Rgb<u8> {
    if *i > 0.0 { lerp(&LAND_LERP, i) } else { lerp(&WATER_LERP, i) }
}

#[derive(Debug, Clone, Copy)]
enum LandmassType {
    FullRandom,
    Peninsula,
    Island,
    Coast,
    Bay,
}

impl TryFrom<u32> for LandmassType {
    type Error = String;
    fn try_from(i: u32) -> Result<LandmassType, Self::Error> {
        match i {
            0 => Ok(LandmassType::FullRandom),
            1 => Ok(LandmassType::Peninsula),
            2 => Ok(LandmassType::Island),
            3 => Ok(LandmassType::Coast),
            4 => Ok(LandmassType::Bay),
            _ => Err("No such landmass type".into()),
        }
    }
}

// scales the perturbation to this amount
const PERTURBATION: f64 = 0.2;

#[tracing::instrument(level = "trace", skip_all, fields(x = x, y = y))]
fn perturb_by_angle(noise_gen: &impl NoiseFn<f64, 1>, x: f64, y: f64) -> f64 {
    if x == 0.0 && y == 0.0 {
        return 1.0;
    }
    let angle = (y.atan2(x) + consts::PI) / consts::TAU; // in turns
    let noise = noise_gen.get([angle]) * PERTURBATION + 1.0;
    trace!(angle, noise);
    noise
}

#[tracing::instrument(level = "trace", skip_all, fields(landmass_type = ?kind))]
fn make_shape_grid(
    width: usize,
    height: usize,
    kind: LandmassType,
    mut rng: impl RngExt,
) -> Result<Grid<f64>, String> {
    // unit type grid allows us to easily calculate some values before we fill a real one
    let geometry_grid = Grid::new_filled(width, height, ())?;
    let (mid_x, mid_y) = geometry_grid.world_coords(geometry_grid.midpoint()).unwrap();
    let (max_x, max_y) = geometry_grid.max_world_coords();
    let max_dist = f64::sqrt((max_x - mid_x).powi(2) + (max_y - mid_y).powi(2));

    let noise_gen = noise::Perlin::new(rng.next_u32());

    let shape_func = |(x, y): (f64, f64)| match kind {
        Island => {
            let dist = f64::sqrt((x - mid_x).powi(2) + (y - mid_y).powi(2));
            let dist = max_dist - dist * perturb_by_angle(&noise_gen, x - mid_x, y - mid_y);
            smootherstep(dist, 0.0, max_dist)
        }
        _ => 1.0,
    };

    let shape_grid = Grid::new_with_world_coords(width, height, shape_func)?;
    Ok(shape_grid)
}

#[tracing::instrument(level = "debug")]
pub fn generate(rng_master: &RngMaster, size: usize) -> Result<Grid<f64>, String> {
    let mut rng = rng_master.for_stage("elevation");
    let noise_seed = rng.next_u32();
    trace!(%noise_seed);
    let scale = rng.d(4);
    let frequency = scale as f64 / size as f64;
    info!(%scale, "Randomly-chosen parameter for elevation noise");

    let fbm = noise::Fbm::<noise::PerlinSurflet>::new(noise_seed).set_frequency(frequency);
    trace!(
        origin = fbm.get([0.0, 0.0]),
        _0_5_0_5 = fbm.get([0.5, 0.5]),
        size_size = fbm.get([size as f64, size as f64]),
        "Noise values at"
    );

    // overall shape of land (island, peninsula, etc) is guided by a "shape grid"
    // of values to multiply the noise-generated elevation by
    let landmass = LandmassType::Island;
    let shape_grid = make_shape_grid(size, size, landmass, &mut rng)?;
    trace!(nans_in_grid = shape_grid.iter().any(|n| n.is_nan()));
    trace!(
        grid_min = shape_grid.iter().copied().fold(f64::INFINITY, f64::min),
        grid_max = shape_grid.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        grid_mean = shape_grid.iter().sum::<f64>() / shape_grid.len() as f64,
    );

    // raising the noise to an exponent e changes overall shape:
    // e < 1 raises bottom, giving rolling highlands
    // e > 1 lowers bottom, giving flat valley bottoms
    // randomly choose between 0.5 and 3.5
    let e = rng.d(7) as f64 / 2.0;
    info!(%e, "Randomly-chosen parameter for elevation redistribution exponent");
    // redblobgames recommends multiplying by a fudge factor first
    // also, we need to normalise to 0 <= n <= 1
    const FUDGE: f64 = 1.2;

    let mut grid = Grid::new_with_index(size, size, |idx| {
        let (x, y) = shape_grid.world_coords(idx).unwrap();
        let n = (normalise(fbm.get([x, y])) * FUDGE).powf(e);
        n * shape_grid[idx]
    })?;

    // instead of picking a height to be sea level, instead choose the fraction
    // of map to be land, and derive sea level from that
    let mut sorted = grid.iter().copied().collect::<Vec<f64>>();
    let land_fraction: f64 = rng.random();
    info!(%land_fraction, "Randomly-chosen parameter for land fraction");
    sorted.sort_by(f64::total_cmp);
    let shore_idx = (sorted.len() as f64 * (1.0 - land_fraction)) as usize;
    let sea_level = sorted[shore_idx];

    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    trace!(min, max, sea_level);

    // normalise so that above sea level goes to +1, below to -1
    for elev in grid.iter_mut() {
        if *elev > sea_level {
            *elev = (*elev - sea_level) / (max - sea_level);
        } else {
            *elev = (*elev - sea_level) / (sea_level - min);
        }
    }

    Ok(grid)
}
