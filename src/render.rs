use std::path::Path;

use crate::hex::Grid;

// expects input in range -1.0 to 1.0, as returned by NoiseFn
#[allow(clippy::cast_possible_truncation)]
fn to_greyscale(i: f64) -> u8 {
    ((i + 1.0) * 128.0).floor() as u8
}

pub fn render_f64_greyscale(
    grid: Grid<f64>,
    seed: &str,
    output_dirname: &str,
) -> Result<(), image::ImageError> {
    let (width, height) = grid.dimensions();
    let image = image::GrayImage::from_raw(
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        grid.iter().map(|i| to_greyscale(*i)).collect(),
    )
    .expect("Grid's store always has exactly width * height elements");

    let file_path = Path::new(output_dirname).join(seed).with_extension("noise.png");
    image.save_with_format(file_path, image::ImageFormat::Png)
}

pub fn render_u8_greyscale(
    grid: Grid<u8>,
    seed: &str,
    output_dirname: &str,
) -> Result<(), image::ImageError> {
    let (width, height) = grid.dimensions();
    let image = image::GrayImage::from_raw(
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        grid.iter().copied().collect(),
    )
    .expect("Grid's store always has exactly width * height elements");

    let file_path = Path::new(output_dirname).join(seed).with_extension("rings.png");
    image.save_with_format(file_path, image::ImageFormat::Png)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_greyscale_min() {
        assert_eq!(to_greyscale(-1.0), 0);
    }

    #[rstest]
    fn test_greyscale_max() {
        assert_eq!(to_greyscale(1.0), 255);
    }
}
