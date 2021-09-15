use std::path::Path;
use std::fs::File;

pub struct MonoImage {
    pub pixels: Box<[u8]>,
    pub width: usize,
    pub height: usize
}

impl MonoImage {
    pub fn new(width: usize, height: usize) -> Self {
        Self { 
            pixels: vec![0; width * height].into(),
            width,
            height
        }
    }

    pub fn offset(&self, x: usize, y: usize) -> usize {
        let x = x.clamp(0, self.width - 1);
        let y = y.clamp(0, self.height - 1);

        y * self.width + x
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, p: u8) {
        self.pixels[self.offset(x, y)] = p;
    }

    pub fn save_png(&self, out: &Path) {
        let output = File::create(out).unwrap();
        let w = std::io::BufWriter::new(output);
    
        let mut enc = 
            png::Encoder::new(w, self.width as u32, self.height as u32);
    
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
    
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(&self.pixels).unwrap();
    }
}