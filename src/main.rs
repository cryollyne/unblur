use std::fmt::Display;

use clap::Parser;
use image::{ImageError, ImageReader};
use num::Complex;
use sized_image::SizedImage;
use vector::Vector;

mod sized_image;
mod vector;

type Cvec4 = Vector<Complex<f32>, 4>;

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
enum Kernel {
    Box,
    Gaussian,
}

impl Display for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = Kernel::Gaussian)]
    kernel: Kernel,
    #[arg(long)]
    kernel_file: Option<String>,

    #[arg(short, long, default_value_t = 1f32)]
    size: f32,

    image: String,
}

fn main() {
    let args = Args::parse();

    let image = get_image(&args.image).expect("failed to read image");

    println!("{image:?}");
}

fn get_image(file: &str) -> Result<SizedImage<Cvec4>, ImageError> {
    let black = Cvec4::default();
    let image = ImageReader::open(file)?.decode()?;
    let (width, height) = (image.width(), image.height());
    let pixels = image
        .to_rgba32f()
        .pixels()
        .map(|x| Cvec4::new(x.0.map(Into::into)))
        .collect::<Vec<_>>();

    Ok(SizedImage::from(width, height, &pixels, black))
}
