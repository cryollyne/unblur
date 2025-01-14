use std::{fmt::Display, num::Wrapping};

use clap::Parser;
use image::{DynamicImage, ImageBuffer, ImageError, ImageReader, Rgba};
use kernel::KernelGenerator;
use num::Complex;
use sized_image::{Domain, SizedImage, TimeDomain};
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

    #[arg(long)]
    no_padding: bool,

    #[arg(short, long, default_value_t = 1.0/256.0)]
    noise_magnitude: f32,

    image: String,
}

fn main() -> Result<(), &'static str> {
    let args = Args::parse();

    let image = get_input_image(&args.image, !args.no_padding)
        .expect("failed to read image");
    let kernel = match args.kernel_file {
        Some(file) => get_image(&file)
            .expect("failed to read kernel file")
            .pad_to_new_size(image.width(), image.height(), Cvec4::default()),
        None => kernel::generate_image(image.width(), image.height(), args.kernel.to_gen(), &args),
    };

    assert_eq!(image.width(), kernel.width());
    assert_eq!(image.height(), kernel.height());

    let mut image = fourier_transform(image)?;
    let mut kernel = fourier_transform(kernel)?;

    // calculate mean power densities
    let noise_mean_power_density = calculate_noise(image.width(), image.height(), args.noise_magnitude)?;
    let signal_mean_power_density = image.pixels.iter()
        .map(|x| {
            x.map(|a| a.norm_sqr().into())
        })
        .fold(Cvec4::new([0.0.into();4]), |acc, x| acc + x);

    for i in 0..kernel.width() {
        for j in 0..kernel.height() {
            let index = kernel.index_of(i,j);
            let px = kernel.pixels[index];
            let px_conj = px.map(|x| x.conj());
            let px_sq_norm = px.map(|x| x.norm_sqr().into());
            kernel.pixels[index] = px_conj * signal_mean_power_density
                / (px_sq_norm*signal_mean_power_density + noise_mean_power_density)
        }
    }

    for (im, krn) in image.pixels.iter_mut().zip(kernel.pixels.iter().copied()) {
        *im = (*im) * krn;
    }

    let image = inverse_fourier_transform(image)?;
    write_image(&args.output, &image)
        .expect("failed to write image");

    Ok(())
}

fn calculate_noise(width: u32, height: u32, noise_magnitude: f32) -> Result<Cvec4, fourier_transform::FourierTransformError> {
    let mut seed = Wrapping(231724u32);
    let mut noise = |i| {
        seed = Wrapping(41)*seed*seed + Wrapping(71)*seed + Wrapping(93);
        seed = Wrapping(37)*seed*seed*Wrapping(i) + Wrapping(337)*seed*Wrapping(i) + Wrapping(80);
        seed = Wrapping(991)*seed*seed + Wrapping(13)*seed + Wrapping(237);
        seed %= 231734;
        ((seed.0 % 128) as f32) / 64.0 - 1.0
    };
    let mut noise_tex = SizedImage::new(width, height, Cvec4::default());
    for (i,px) in noise_tex.pixels.iter_mut().enumerate() {
        *px = Cvec4::new([
            (noise_magnitude * noise(4*i as u32)).into(),
            (noise_magnitude * noise(4*i as u32+1)).into(),
            (noise_magnitude * noise(4*i as u32+2)).into(),
            (noise_magnitude * noise(4*i as u32+3)).into()
        ]);
    }

    let noise_tex = fourier_transform(noise_tex)?;
    Ok(noise_tex.pixels.iter()
        .map(|x| {
            x.map(|a| a.norm_sqr().into())
        })
        .fold(Cvec4::new([0.0.into();4]), |acc, x| acc + x))
}

fn get_input_image(file: &str, pad_image: bool) -> Result<SizedImage<Cvec4, TimeDomain>, ImageError> {
    let i = get_image(file)?;
    let w = i.width();
    let h = i.height();
    let padding_factor = if pad_image { 2 } else { 1 };
    let mut i = i.pad_to_new_size(padding_factor*w, padding_factor*h, Cvec4::default());
    for px in i.pixels.iter_mut() {
        *px = px.map(|x| {
            if x.re < 0.01 {
                0.01.into()
            } else {x}
        })
    }
    Ok(i)
}

fn write_image<D: Domain>(file: &str, image: &SizedImage<Cvec4, D>) -> Result<(), ImageError> {
    let mut new_image = ImageBuffer::<Rgba<f32>, Vec<_>>::new(image.width(), image.height());
    let px = image.pixels.iter().flat_map(|x| {
        let mut a = x.data.map(|x| x.re);
        a[3] = 1.0;
        a
    });
    new_image.iter_mut().zip(px).for_each(|(a, b)| *a = b);
    DynamicImage::from(new_image).to_rgba16().save(file)
}

fn get_image<D: Domain>(file: &str) -> Result<SizedImage<Cvec4, D>, ImageError> {
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
