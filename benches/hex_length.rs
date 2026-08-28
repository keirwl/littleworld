use littleworld::hex::HexAxial;

fn main() {
    divan::main();
}

#[divan::bench]
fn max(bencher: divan::Bencher) {
    let a = HexAxial { q: 3, r: -7 };
    bencher.bench(|| divan::black_box(a).length_max());
}

#[divan::bench]
fn div(bencher: divan::Bencher) {
    let a = HexAxial { q: 3, r: -7 };
    bencher.bench(|| divan::black_box(a).length());
}
