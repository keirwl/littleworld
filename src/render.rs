use std::path::Path;

use image::{ImageError, Rgb, RgbImage};
use tracing::trace;

use crate::hex::Grid;
use crate::util::{SQRT_3, normalise};

#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn to_greyscale(i: &f64) -> Rgb<u8> {
    let b = (normalise(*i) * 255.0).round() as u8;
    Rgb([b, b, b])
}

fn pixel_image<T, F>(grid: &Grid<T>, colour_map: F, scale: u32) -> Result<RgbImage, ImageError>
where
    F: Fn(&T) -> Rgb<u8>,
{
    let (width, height) = grid.dimensions();
    let mut image = RgbImage::new(width * scale, height * scale);

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

fn hex_image<T, F>(grid: &Grid<T>, colour_map: F, scale: u32) -> Result<RgbImage, ImageError>
where
    F: Fn(&T) -> Rgb<u8>,
{
    let (grid_w, grid_h) = grid.dimensions();
    let r = scale as f64 / 2.0; // taking scale as hex width (corner to corner)
    // image dimensions account for size from hex packing
    let image_w = (1.5 * r * (grid_w as f64 - 1.0) + 2.0 * r).ceil() as u32;
    let image_h = (SQRT_3 * r * (grid_h as f64 + 0.5)).ceil() as u32;
    trace!(grid_w, grid_h, image_w, image_h);
    let mut image = RgbImage::from_pixel(image_w, image_h, Rgb([0, 0, 0]));

    for ((x, y), t) in grid.iter_world_coords() {
        let colour = colour_map(t);
        let centre_x = x * r + r;
        let centre_y = y * r + r * SQRT_3 / 2.0;

        // the bounding rectangle that the hex occupies: each pixel
        // in it will be checked if it's part of the hex or not
        let bottom = (centre_y + r * SQRT_3 / 2.0).ceil() as u32;
        let top = (centre_y - r * SQRT_3 / 2.0).floor() as u32;
        let left = (centre_x - r).floor() as u32;
        let right = (centre_x + r).ceil() as u32;

        for py in top..bottom {
            for px in left..right {
                let offset_x = px as f64 + 0.5 - centre_x;
                let offset_y = py as f64 + 0.5 - centre_y;
                if offset_y.abs() <= r * SQRT_3 / 2.0
                    && offset_x.abs() + offset_y.abs() / SQRT_3 <= r
                {
                    image.put_pixel(px, py, colour);
                }
            }
        }
    }
    Ok(image)
}

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Pixel(u32),
    Hex(u32),
}

// #[tracing::instrument(level = "debug", skip_all, fields(name = ?name, colour_map_name = std::any::type_name::<F>(), format = ?render_format))]
pub fn render<T, F>(
    grid: &Grid<T>,
    colour_map: F,
    render_format: Format,
    out_dir: &Path,
    name: &Path,
) -> Result<(), ImageError>
where
    F: Fn(&T) -> Rgb<u8>,
{
    let image = match render_format {
        Format::Pixel(scale) => pixel_image(grid, colour_map, scale)?,
        Format::Hex(scale) => hex_image(grid, colour_map, scale)?,
    };
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
    fn test_greyscale_mid() {
        assert_eq!(to_greyscale(&0.0), Rgb([128, 128, 128]));
    }

    #[rstest]
    fn test_greyscale_max() {
        assert_eq!(to_greyscale(&1.0), Rgb([255, 255, 255]));
    }
}
