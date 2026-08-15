#![allow(dead_code)]
use std::{fs::File, io::BufRead};

use medieval::hex;

use rand::prelude::*;

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
    println!("Number of words: {}", words.len());
    let idx = rand::rng().random_range(0..words.len());
    words[idx].clone()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: &str = if args.len() > 1 {
        &args[1]
    } else {
        &get_seed_word()
    };
    let grid: hex::Grid<i32> = hex::Grid::new_filled(1, 2, 0).unwrap();
    println!("{grid:#?}");
    let rnger = medieval::rng::RngMaster::new(seed);
    let mut rng = rnger.for_stage("test");
    println!("{:#?}", rng.get_seed());
    println!("{:#?}", rng.random_bool(0.5));
}

fn d(s: u32) -> u32 {
    rand::rng().random_range(1..=s)
}

fn dn(n: u32, d: u32) -> u32 {
    (0..n).map(|_| rand::rng().random_range(1..=d)).sum()
}
