use std::marker::PhantomData;

pub trait Domain {}
pub struct TimeDomain;
impl Domain for TimeDomain {}
pub struct FrequencyDomain;
impl Domain for FrequencyDomain {}

#[derive(Debug, Clone)]
pub struct SizedImage<Pixel, D: Domain> {
    pub width_log_2: u32,
    pub height_log_2: u32,
    pub pixels: Vec<Pixel>,
    _d: PhantomData<D>,
}

fn log2u32(mut i: u32) -> u32 {
    assert_ne!(i, 0);
    let mut size = 0;
    while i != 1 {
        i >>= 1;
        size += 1;
    }
    size
}

impl<Pixel, D: Domain> SizedImage<Pixel, D> {
    pub fn new(width: u32, height: u32, init: Pixel) -> SizedImage<Pixel, D>
    where
        Pixel: Clone,
    {
        let width_log_2 = log2u32(width);
        let height_log_2 = log2u32(height);
        SizedImage {
            width_log_2,
            height_log_2,
            pixels: vec![init; (1 << width_log_2) * (1 << height_log_2)],
            _d: PhantomData{}
        }
    }

    pub fn convert<New: Domain>(self) -> SizedImage<Pixel, New> {
        SizedImage {
            width_log_2: self.width_log_2,
            height_log_2: self.height_log_2,
            pixels: self.pixels,
            _d: PhantomData{}
        }
    }

    pub fn from(width: u32, height: u32, pixels: &[Pixel], init: Pixel) -> SizedImage<Pixel, D>
    where
        Pixel: Clone,
    {
        let mut s = SizedImage::new(width, height, init);
        s.blit(width, height, pixels);
        s
    }

    pub fn index_of(&self, x: u32, y: u32) -> usize {
        ((1 << self.width_log_2) * y + x) as usize
    }

    pub fn width(&self) -> u32 {
        1 << self.width_log_2
    }
    pub fn height(&self) -> u32 {
        1 << self.height_log_2
    }
    pub fn store_data(&mut self, x: u32, y: u32, pixel: Pixel) {
        let index = self.index_of(x, y);
        self.pixels[index] = pixel;
    }

    fn blit(&mut self, width: u32, height: u32, pixels: &[Pixel])
    where
        Pixel: Clone,
    {
        for i in 0..height {
            for j in 0..width {
                let index = self.index_of(j, i);
                self.pixels[index] = pixels[(i * width + j) as usize].clone();
            }
        }
    }

    pub fn pad_to_new_size(&self, width: u32, height: u32, init: Pixel) -> SizedImage<Pixel, D>
    where
        Pixel: Clone,
    {
        let mut new_image = SizedImage::new(width, height, init);
        new_image.blit(self.width(), self.height(), &self.pixels);
        new_image
    }
}
