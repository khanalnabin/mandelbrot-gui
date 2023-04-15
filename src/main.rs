extern crate image;
extern crate num;

use macroquad::prelude::*;

use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num::Complex;
use std::fs::File;


fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };

    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );

    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 100),
            (25, 75),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex { re: -0.5, im: -0.5 }
    );
}

fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);
    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            }
        }
    }
}

fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PngEncoder::new(output);
    let _res = encoder.write_image(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8);
    Ok(())
}
fn parallel(
    filename: &str,
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    let threads = 100;
    let rows_per_band = bounds.1 / threads + 1;

    let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = rows_per_band * i;
            let height = band.len() / bounds.0;

            let band_bounds = (bounds.0, height);
            let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);
            spawner.spawn(move |_| {
                render(band, band_bounds, band_upper_left, band_lower_right);
            });
        }
    })
    .unwrap();
    write_image(filename, &pixels, bounds).expect("error writing png file");
}

fn redraw() {}
#[macroquad::main("BasicShapes")]
async fn main() {
    let width = 800;
    let height = 600;
    let bounds = (width, height);
    let filename = "mandelbrot.png";
    let mut pixels = vec![0; bounds.0 * bounds.1];
    let mut upper_left = Complex { re: -1.6, im: 1.2 };
    let mut lower_right = Complex { re: 1.6, im: -1.2 };

    parallel(filename, &mut pixels, bounds, upper_left, lower_right);

    let mut texture = load_texture("mandelbrot.png").await.unwrap();

    loop {
        clear_background(WHITE);
        let (mouse_x, mouse_y) = mouse_position();
        let (_, mouse_wheel_y) = mouse_wheel();
        if mouse_wheel_y == 1.0 {
            let current_pos = Complex{ re: (upper_left.re+ (lower_right.re- upper_left.re) / width as f64 * mouse_x as f64), im: (upper_left.im+ (lower_right.im- upper_left.im) / height as f64* mouse_y as f64) };
            upper_left = upper_left + (current_pos - upper_left)/2 as f64;
            lower_right = lower_right -  (lower_right - current_pos)/2 as f64;

            parallel(filename, &mut pixels, bounds, upper_left, lower_right);
            println!("Zoom In!!!");
            println!("Upper Left: {:?} Lower Right: {:?}", upper_left, lower_right);
            texture = load_texture("mandelbrot.png").await.unwrap();
        } else if mouse_wheel_y == -1.0 {
            let current_pos = Complex{ re: (upper_left.re+ (lower_right.re- upper_left.re) / width as f64 * mouse_x as f64), im: (upper_left.im+ (lower_right.im- upper_left.im) / height as f64* mouse_y as f64) };
            upper_left = upper_left - (current_pos - upper_left)/2 as f64;
            lower_right = lower_right +  (lower_right - current_pos)/2 as f64;

            println!("Zoom Out!!!");
            println!("Upper Left: {:?} Lower Right: {:?}", upper_left, lower_right);
            parallel(filename, &mut pixels, bounds, upper_left, lower_right);
            texture = load_texture("mandelbrot.png").await.unwrap();
        }
        draw_texture(
            texture,
            screen_width() / 2. - texture.width() / 2.,
            screen_height() / 2. - texture.height() / 2.,
            WHITE,
        );
        next_frame().await;
    }
}
