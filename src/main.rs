use std::fmt::Display;

use clap::Parser;
use image::{ImageError, ImageReader, Rgba};
use sized_image::SizedImage;

mod sized_image;

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

fn get_image(file: &str) -> Result<SizedImage<Rgba<f32>>, ImageError> {
    let black = Rgba::<f32>::from([0f32, 0f32, 0f32, 0f32]);
    let image = ImageReader::open(file)?.decode()?;
    let (width, height) = (image.width(), image.height());
    let pixels = image.to_rgba32f().pixels().copied().collect::<Vec<_>>();

    Ok(SizedImage::<Rgba<f32>>::from(width, height, &pixels, black))
}
