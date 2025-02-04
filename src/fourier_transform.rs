use std::num::NonZero;

use num::Complex;

use crate::{sized_image::{Domain, FrequencyDomain, SizedImage, TimeDomain}, Cvec4};

#[repr(u8)]
#[derive(Debug)]
pub enum FourierTransformError {
    OutOfMemory = 1,
    InvalidDimension = 2,
    InvalidThreadCount = 3,
}
impl TryFrom<u8> for FourierTransformError {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::OutOfMemory),
            2 => Ok(Self::InvalidDimension),
            3 => Ok(Self::InvalidThreadCount),
            _ => Err(())
        }
    }
}

impl From<FourierTransformError> for &'static str {
    fn from(value: FourierTransformError) -> Self {
        type E =  FourierTransformError;
        match value {
            E::OutOfMemory => "out of memory",
            E::InvalidDimension => "invalid dimension",
            E::InvalidThreadCount => "invalid thread count",
        }
    }
}

pub fn fourier_transform(mut image: SizedImage<Cvec4, TimeDomain>) -> Result<SizedImage<Cvec4, FrequencyDomain>, FourierTransformError> {
    fourier_transform_raw(&mut image, false)?;
    Ok(image.convert::<FrequencyDomain>())
}

pub fn inverse_fourier_transform(mut image: SizedImage<Cvec4, FrequencyDomain>) -> Result<SizedImage<Cvec4, TimeDomain>, FourierTransformError> {
    fourier_transform_raw(&mut image, true)?;
    Ok(image.convert::<TimeDomain>())
}

fn fourier_transform_raw<D: Domain>(image: &mut SizedImage<Cvec4, D>, inverse: bool) -> Result<(),FourierTransformError> {
    let ptr = image.pixels.as_mut_ptr() as _;
    let threads = std::thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1) as _;

    let res = unsafe {
        fourier_transform_c_ffi(ptr, image.width_log_2, image.height_log_2, threads, inverse)
    };
    match FourierTransformError::try_from(res) {
        Ok(err) => Err(err),
        Err(()) => Ok(())
    }
}

extern "C" {
    #[link_name = "fft2d"]
    fn fourier_transform_c_ffi(
        data: *mut Complex<f32>,
        logWidth: u32,
        logHeight: u32,
        threads: u32,
        inverse: bool,
    ) -> u8;
}
