#![allow(dead_code)]
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader};
use std::path::Path;

use argh::FromArgs;
use littleworld::generation::elevation::{colour_land_sea, generate};
use littleworld::render::{Format, render, to_greyscale};
use littleworld::rng::RngMaster;
use rand::prelude::*;
use tracing::{error, info};
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

fn get_seed_hex() -> String {
    let seed = rand::rng().next_u64();
    format!("{:x?}", seed)
}

fn get_seed_word() -> String {
    let word_file = File::open("/usr/share/dict/words");
    if word_file.is_err() {
        return get_seed_hex();
    }
    let words: Vec<String> = BufReader::new(word_file.unwrap())
        .lines()
        .map_while(Result::ok)
        .filter(|l| *l == l.to_lowercase())
        .filter(|l| l.len() > 3)
        .filter(|l| !l.contains(['-']))
        .collect();
    let idx = rand::rng().random_range(0..words.len());
    words[idx].clone()
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
    #[argh(option, default = "8")]
    scale: u32,
    /// output directory
    #[argh(option, default = "String::from(\"output/\")")]
    out_dir: String,
    /// print square grid pixel output
    #[argh(option, short = 'p', default = "false")]
    pixel: bool,
}

fn main() {
    let config: Config = argh::from_env();
    let rng_master = RngMaster::new(&config.seed);
    let mut _test_rng = rng_master.for_stage("test");

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

    let print_format = if config.pixel {
        Format::Pixel(config.scale)
    } else if config.scale < 8 {
        error!("Will lose information drawing hexes smaller than 8px");
        return;
    } else {
        Format::Hex(config.scale)
    };

    info!(master_seed = config.seed, size = config.size, "Starting run");

    let elevation_grid = generate(&rng_master, config.size).unwrap();
    render(
        &elevation_grid,
        to_greyscale,
        print_format,
        &run_dir_path,
        Path::new("elevation_greyscale"),
    )
    .unwrap();
    render(&elevation_grid, colour_land_sea, print_format, &run_dir_path, Path::new("elevation"))
        .unwrap();
}

#[cfg(test)]
mod tests {
    use littleworld::{hex, rng};
    use noise::{Fbm, MultiFractal, NoiseFn};
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
        let mut rng = RngMaster::new(GOLDEN_SEED).for_stage(GOLDEN_STAGE);
        let fbm = Fbm::<noise::PerlinSurflet>::new(rng.next_u32())
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
