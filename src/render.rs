use std::path::Path;

use image::{ImageError, Rgb, RgbImage};

use crate::hex::Grid;

// expects input in range -1.0 to 1.0, as returned by NoiseFn
#[allow(clippy::cast_possible_truncation)]
pub fn to_greyscale(i: &f64) -> Rgb<u8> {
    let b = ((i + 1.0) * 128.0).floor() as u8;
    Rgb([b, b, b])
}

#[derive(Debug)]
pub enum Format {
    Pixel,
    Hex(u32),
}

fn pixel_image<T, F>(grid: &Grid<T>, colour_map: F, scale: u32) -> Result<RgbImage, ImageError>
where
    F: Fn(&T) -> Rgb<u8>,
{
    let (width, height) = grid.dimensions();
    let mut image = image::RgbImage::new(width * scale, height * scale);

    for ((col, row), t) in grid.iter_coords() {
        let pixel = colour_map(t);
        for dx in 0..scale {
            for dy in 0..scale {
                image.put_pixel(col * scale + dx, row * scale + dy, pixel);
            }
        }
    }
    Ok(image)
}

#[tracing::instrument(skip(grid, colour_map))]
pub fn render<T, F>(
    grid: &Grid<T>,
    colour_map: F,
    scale: u32,
    out_dir: &Path,
    name: &Path,
) -> Result<(), ImageError>
where
    T: std::fmt::Debug,
    F: Fn(&T) -> Rgb<u8>,
{
    let _render_format = Format::Pixel;
    let image = pixel_image(grid, colour_map, scale)?;
    let file_path = out_dir.join(name).with_extension("png");
    image.save_with_format(file_path, image::ImageFormat::Png)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_greyscale_min() {
        assert_eq!(to_greyscale(&-1.0), Rgb([0, 0, 0]));
    }

    #[rstest]
    fn test_greyscale_max() {
        assert_eq!(to_greyscale(&1.0), Rgb([255, 255, 255]));
    }
}
