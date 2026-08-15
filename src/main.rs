#![allow(dead_code)]
use argh::FromArgs;
use medieval::render;
use noise::{MultiFractal, NoiseFn};
use rand::{prelude::*, rngs::ChaCha8Rng};
use std::{fs::File, io::BufRead};

use medieval::hex;

struct Realm {
    size: u32,
    density: u32,
    population: u32,
    cities: Vec<u32>,
    num_towns: u32,
}

/// procgen
#[derive(FromArgs)]
struct Config {
    /// master seed
    #[argh(option, default = "get_seed_word()")]
    seed: String,
    /// size of grid (will be square)
    #[argh(option, default = "512")]
    size: usize,
    /// output directory
    #[argh(option, default = "String::from(\"output/\")")]
    out_dir: String,
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

fn noise_grid(config: &Config, mut rng: ChaCha8Rng) -> hex::Grid<f64> {
    let noise_seed = rng.next_u32();
    println!("Perlin seed: {}", noise_seed);
    let scale = rng.random_range(1..=20);
    let frequency = scale as f64 / config.size as f64;
    println!(
        "Frequency scale: {}, giving frequency: {}",
        scale, frequency
    );

    let fbm = noise::Fbm::<noise::PerlinSurflet>::new(noise_seed).set_frequency(frequency);
    println!(
        "Perlin at (0.5, 0.5): {}, (size, size): {}",
        fbm.get([0.5, 0.5]),
        fbm.get([config.size as f64, config.size as f64])
    );
    let grid = hex::Grid::new_with_coords(config.size, config.size, |(col, row)| {
        fbm.get([col as f64, row as f64])
    })
    .unwrap();
    println!(
        "Grid of size {} created with seed '{}'",
        config.size, &config.seed
    );
    println!("NaNs in grid? {}", grid.iter().any(|n| n.is_nan()));
    let grid_min = grid.iter().copied().fold(f64::INFINITY, f64::min);
    let grid_max = grid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let grid_mean = grid.iter().sum::<f64>() / grid.len() as f64;
    println!(
        "Grid min: {:.3}, max: {:.3}, mean: {:.3}",
        grid_min, grid_max, grid_mean
    );
    grid
}

fn ring_grid(config: &Config) -> hex::Grid<u8> {
    let mut grid = hex::Grid::<u8>::new_filled(config.size, config.size, 0).unwrap();
    let middle_idx = (config.size * config.size / 2) + (config.size / 2 - 1);
    for n in 0..8 {
        let colour: u8 = if n == 0 { 0 } else { (32 * n) - 1 };
        for i in grid.ring(middle_idx, n as u32) {
            grid.set(i, colour).unwrap();
        }
    }
    grid
}

fn main() {
    let config: Config = argh::from_env();
    let rnger = medieval::rng::RngMaster::new(&config.seed);
    let rng = rnger.for_stage("m0");
    let n_grid = noise_grid(&config, rng);
    render::render_f64_greyscale(n_grid, &config.seed, &config.out_dir).unwrap();

    let r_grid = ring_grid(&config);
    render::render_u8_greyscale(r_grid, &config.seed, &config.out_dir).unwrap();
}

fn d(s: u32) -> u32 {
    rand::rng().random_range(1..=s)
}

fn dn(n: u32, d: u32) -> u32 {
    (0..n).map(|_| rand::rng().random_range(1..=d)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use medieval::rng;
    use rstest::rstest;

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
        let bytes = grid
            .iter()
            .flat_map(|f| f.to_be_bytes())
            .collect::<Vec<u8>>();
        let hash = rng::hash_fnv_1a(&bytes);
        println!("Golden hash: {:x}", hash);
        assert_eq!(GOLDEN_HASH, hash)
    }
}
