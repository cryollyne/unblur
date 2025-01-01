#[derive(Debug, Clone)]
pub struct SizedImage<Pixel> {
    width_log_2: u32,
    height_log_2: u32,
    pixels: Vec<Pixel>,
}

fn log2u32(mut i: u32) -> u32 {
    let mut size = 0;
    while i > 0 {
        i >>= 1;
        size += 1;
    }
    size
}

impl<Pixel> SizedImage<Pixel> {
    pub fn new(width: u32, height: u32, init: Pixel) -> SizedImage<Pixel>
    where
        Pixel: Clone,
    {
        let width_log_2 = log2u32(width);
        let height_log_2 = log2u32(height);
        println!("width_log_2: {width_log_2}, height_log_2: {height_log_2}");
        println!("width: {}, height: {}", 1 << width_log_2, 1 << height_log_2);
        SizedImage {
            width_log_2,
            height_log_2,
            pixels: vec![init; (1 << width_log_2) * (1 << height_log_2)],
        }
    }

    pub fn from(width: u32, height: u32, pixels: &[Pixel], init: Pixel) -> SizedImage<Pixel>
    where
        Pixel: Clone,
    {
        let mut s = SizedImage::new(width, height, init);
        for i in 0..height {
            for j in 0..width {
                let index = s.index_of(j, i);
                s.pixels[index] = pixels[(i * width + j) as usize].clone();
            }
        }

        s
    }

    pub fn index_of(&self, x: u32, y: u32) -> usize {
        ((1 << self.width_log_2) * y + x) as usize
    }
}
