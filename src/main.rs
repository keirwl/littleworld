#![allow(dead_code)]
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::BufRead;
use std::path::Path;

use argh::FromArgs;
use medieval::generation::elevation::{brown_palette, brownscale, generate};
use medieval::hex;
use medieval::render::{render, to_greyscale};
use medieval::rng::Dice;
use noise::{MultiFractal, NoiseFn};
use rand::prelude::*;
use rand::rngs::ChaCha8Rng;
use tracing::{Level, event};
use tracing_subscriber::fmt::format::{FmtSpan, PrettyFields};
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

struct Realm {
    size: u32,
    density: u32,
    population: u32,
    cities: Vec<u32>,
    num_towns: u32,
}

fn get_seed_word() -> String {
    let word_file = std::io::BufReader::new(File::open("/usr/share/dict/linux.words").unwrap());
    let words: Vec<String> = word_file
        .lines()
        .map_while(Result::ok)
        .filter(|l| *l == l.to_lowercase())
        .filter(|l| l.len() > 3)
        .filter(|l| !l.contains(['-']))
        .collect();
    let idx = rand::rng().random_range(0..words.len());
    words[idx].clone()
}

#[tracing::instrument]
fn noise_grid(config: &Config, mut rng: ChaCha8Rng) -> hex::Grid<f64> {
    let noise_seed = rng.next_u32();
    event!(Level::TRACE, %noise_seed);
    let scale = rng.d(20);
    let frequency = f64::from(scale) / config.size as f64;
    event!(Level::TRACE, %scale, %frequency);

    let fbm = noise::Fbm::<noise::PerlinSurflet>::new(noise_seed).set_frequency(frequency);
    event!(
        Level::TRACE,
        origin = fbm.get([0.0, 0.0]),
        _0_5_0_5 = fbm.get([0.5, 0.5]),
        size_size = fbm.get([config.size as f64, config.size as f64]),
        "Noise values at"
    );
    let grid = hex::Grid::new_with_coords(config.size, config.size, |(col, row)| {
        fbm.get([col as f64, row as f64])
    })
    .unwrap();
    event!(Level::TRACE, config.size, config.seed);
    event!(Level::TRACE, nans_in_grid = grid.iter().any(|n| n.is_nan()));
    let grid_min = grid.iter().copied().fold(f64::INFINITY, f64::min);
    let grid_max = grid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let grid_mean = grid.iter().sum::<f64>() / grid.len() as f64;
    event!(Level::TRACE, grid_min, grid_max, grid_mean);
    grid
}

fn ring_grid(config: &Config) -> hex::Grid<u8> {
    let mut grid = hex::Grid::<u8>::new_filled(config.size, config.size, 0).unwrap();
    let middle_idx = (config.size * config.size / 2) + (config.size / 2 - 1);
    for n in 0..8 {
        let colour: u8 = if n == 0 { 0 } else { (32 * n) - 1 };
        for i in grid.ring(middle_idx, u32::from(n)) {
            grid.set(i, colour).unwrap();
        }
    }
    grid
}

/// Procedural generation of a medieval fantasy world
#[derive(Debug, FromArgs)]
struct Config {
    /// master seed
    #[argh(option, default = "get_seed_word()")]
    seed: String,
    /// size of grid (will be square)
    #[argh(option, default = "512")]
    size: usize,
    /// image scale
    #[argh(option, default = "1")]
    scale: u32,
    /// output directory
    #[argh(option, default = "String::from(\"output/\")")]
    out_dir: String,
}

fn main() {
    let config: Config = argh::from_env();
    let rng_master = medieval::rng::RngMaster::new(&config.seed);

    let run_dir_path = Path::new(&config.out_dir).join(&config.seed);
    create_dir_all(&run_dir_path).unwrap();
    let run_log_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(run_dir_path.join("run.log"))
        .unwrap();

    tracing_subscriber::registry()
        .with(
            // stdout layer, timestamps have no date
            fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(false)
                .with_target(false)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::UtcTime::new(
                    time::macros::format_description!("[hour]:[minute]:[second].[subsecond]"),
                )),
        )
        .with(
            // per-run file layer, no ansi colour codes
            fmt::layer()
                .fmt_fields(PrettyFields::new())
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(false)
                .with_target(false)
                .with_ansi(false)
                .compact()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(run_log_file),
        )
        .with(EnvFilter::from_default_env())
        .init();

    let elevation_grid = generate(rng_master, config.size).unwrap();
    render(
        &elevation_grid,
        to_greyscale,
        config.scale,
        &run_dir_path,
        Path::new("elevation_greyscale"),
    )
    .unwrap();
    render(
        &elevation_grid,
        brown_palette,
        config.scale,
        &run_dir_path,
        Path::new("elevation_brown_palette"),
    )
    .unwrap();
    render(
        &elevation_grid,
        brownscale,
        config.scale,
        &run_dir_path,
        Path::new("elevation_brown_ramp"),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use medieval::rng;
    use rstest::rstest;

    use super::*;

    // Checks that staged seeding, random number generation and grid are stable across updates.
    // Using a master seed of "Urist" (in honour of Dwarf Fortress), a stage name of "0", and a
    // size of 256, a PerlinSurflet fBm with default parameters, except frequency equal to 1/size,
    // over the grid gives a FNV-1a hash of:
    // aff27af64ef26a64
    //
    #[rstest]
    fn golden_hash() {
        const GOLDEN_SEED: &str = "Urist";
        const GOLDEN_STAGE: &str = "0";
        const GOLDEN_SIZE: usize = 256;
        const GOLDEN_HASH: u64 = 0xaff27af64ef26a64;
        let mut rng = medieval::rng::RngMaster::new(GOLDEN_SEED).for_stage(GOLDEN_STAGE);
        let fbm = noise::Fbm::<noise::PerlinSurflet>::new(rng.next_u32())
            .set_frequency(1.0 / GOLDEN_SIZE as f64);
        let grid = hex::Grid::new_with_coords(GOLDEN_SIZE, GOLDEN_SIZE, |(col, row)| {
            fbm.get([col as f64, row as f64])
        })
        .unwrap();
        // break the grid to check test:
        // grid.set(10, 5.0);
        let bytes = grid.iter().flat_map(|f| f.to_be_bytes()).collect::<Vec<u8>>();
        let hash = rng::hash_fnv_1a(&bytes);
        println!("Golden hash: {hash:x}");
        assert_eq!(GOLDEN_HASH, hash);
    }
}
