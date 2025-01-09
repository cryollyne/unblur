use std::fmt::Display;

use clap::Parser;
use image::{DynamicImage, ImageBuffer, ImageError, ImageReader, Rgba};
use kernel::KernelGenerator;
use num::Complex;
use sized_image::{SizedImage, TimeDomain};
use vector::Vector;
use fourier_transform::{fourier_transform, inverse_fourier_transform};

mod fourier_transform;
mod kernel;
mod sized_image;
mod vector;

type Cvec4 = Vector<Complex<f32>, 4>;

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
enum Kernel {
    Box,
    Gaussian,
}

impl Kernel {
    fn to_gen(self) -> Box<dyn KernelGenerator> {
        match self {
            Kernel::Box => Box::new(kernel::BoxKernelGen {}),
            Kernel::Gaussian => Box::new(kernel::GaussianKernelGen {}),
        }
    }
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

    #[arg(short, long, default_value_t = String::from("output.png"))]
    output: String,

    image: String,
}

fn main() -> Result<(), &'static str> {
    let args = Args::parse();

    let image = {
        let i = get_image(&args.image).expect("failed to read image");
        let w = i.width();
        let h = i.height();
        i.pad_to_new_size(w * 2, h * 2, Cvec4::default())
    };
    let kernel = match args.kernel_file {
        Some(file) => get_image(&file)
            .expect("failed to read kernel file")
            .pad_to_new_size(image.width(), image.height(), Cvec4::default()),
        None => kernel::generate_image(image.width(), image.height(), args.kernel.to_gen(), &args),
    };

    assert_eq!(image.width(), kernel.width());
    assert_eq!(image.height(), kernel.height());

    let mut image = fourier_transform(image)?;
    let kernel = fourier_transform(kernel)?;

    for (im, krn) in image.pixels.iter_mut().zip(kernel.pixels.iter()) {
        *im = (*im) / (*krn);
    }

    let image = inverse_fourier_transform(image)?;

    write_image(&args.output, image.width(), image.height(), &image.pixels)
        .expect("failed to write image");

    Ok(())
}

fn write_image(file: &str, width: u32, height: u32, pixels: &[Cvec4]) -> Result<(), ImageError> {
    let mut image = ImageBuffer::<Rgba<f32>, Vec<_>>::new(width, height);
    let px = pixels.iter().flat_map(|x| x.data.map(|x| x.re));
    image.iter_mut().zip(px).for_each(|(a, b)| *a = b);
    DynamicImage::from(image).to_rgba8().save(file)
}

fn get_image(file: &str) -> Result<SizedImage<Cvec4, TimeDomain>, ImageError> {
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
