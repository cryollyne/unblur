use num::abs;

use crate::{sized_image::SizedImage, Args, Cvec4};

#[derive(Debug)]
pub struct KernelConfig {
    size: f32,
}

pub trait KernelGenerator {
    fn sample(&self, x: i32, y: i32, config: &KernelConfig) -> Cvec4;
}

pub struct BoxKernelGen;
impl KernelGenerator for BoxKernelGen {
    fn sample(&self, x: i32, y: i32, config: &KernelConfig) -> Cvec4 {
        let size = config.size as i32;
        let scaling = 1f32 / ((2 * size - 1) * (2 * size - 1)) as f32;
        if abs(x) < size && abs(y) < size {
            Cvec4::new([scaling.into(); 4])
        } else {
            Cvec4::default()
        }
    }
}

pub struct GaussianKernelGen;
impl KernelGenerator for GaussianKernelGen {
    fn sample(&self, x: i32, y: i32, config: &KernelConfig) -> Cvec4 {
        let pi = std::f32::consts::PI;
        let sigma = config.size;
        let p0 = 1f32 / (2f32 * pi * sigma * sigma);
        let r = (x * x + y * y) as f32;
        let weight = p0 * f32::exp(-r / (2f32 * sigma * sigma));
        let px = [weight.into(), weight.into(), weight.into(), 1f32.into()];
        Cvec4::new(px)
        // Cvec4::new([weight.into(); 4])
    }
}

pub fn generate_image(
    width: u32,
    height: u32,
    generator: Box<dyn KernelGenerator>,
    args: &Args,
) -> SizedImage<Cvec4> {
    let config = KernelConfig { size: args.size };

    let mut kernel = SizedImage::new(width, height, Cvec4::default());
    let w = kernel.width() as i32;
    let h = kernel.height() as i32;
    for i in 0..w {
        for j in 0..h {
            let x = i - w / 2;
            let y = j - h / 2;
            kernel.store_data(i as u32, j as u32, generator.sample(x, y, &config));
        }
    }
    kernel
}
