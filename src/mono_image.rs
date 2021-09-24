use std::path::Path;
use std::fs::File;

use png::OutputInfo;
use crate::program_cpu::*;

pub struct MonoImage {
    pub pixels: Vec<u8>,
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

    pub fn load_from_file(png: &str) -> Self {
        let (frame_info, mut buf) = Self::load_png_pixels(png);
    
        buf.resize(frame_info.buffer_size(), 0);

        let strip = |buf: &[u8], stride| {
            let mut img = 
                MonoImage::new(
                    frame_info.width as usize, 
                    frame_info.height as usize);
            
            rgba_to_grayscale(&buf, &mut img.pixels, stride);
            img
        };
    
        match frame_info.color_type {
            | png::ColorType::Grayscale 
            | png::ColorType::GrayscaleAlpha => 
                Self { 
                    pixels: buf,
                    width: frame_info.width as usize,
                    height: frame_info.height as usize
                },
            | png::ColorType::Rgb => strip(&buf, 3),
            | png::ColorType::Rgba => strip(&buf, 4),
            | _ => panic!("PNG frame must in grayscale/rgb type.")
        }
    }

    pub fn offset(&self, x: usize, y: usize) -> usize {
        let x = x.clamp(0, self.width - 1);
        let y = y.clamp(0, self.height - 1);

        y * self.width + x
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, p: u8) {
        let offset = self.offset(x, y);
        self.pixels[offset] = p;
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.pixels.resize(width * height, 0);
        self.width = width;
        self.height = height;
    }

    pub fn clear_color(&mut self) {
        for i in &mut self.pixels {
            *i = 0;
        }
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

    pub fn edge_detect(&self, to: &mut MonoImage) {
        assert!(to.width == self.width);
        assert!(to.height == self.height);

        edge_detect(&self.pixels, &mut to.pixels, to.width, to.height);
    }

    pub fn edge_generate_sdf(&self, to: &mut MonoImage, stride: usize, search_radius: usize) {
        assert!(to.width == self.width / stride);
        assert!(to.height == self.height / stride);

        sdf_generate(
            &self.pixels, 
            &mut to.pixels, 
            self.width, 
            self.height, 
            to.width, 
            to.height, 
            stride, 
            search_radius)
    }

    pub fn load_png_pixels(png: &str) -> (OutputInfo, Vec<u8>) {
        let input_png = 
                png::Decoder::new(
                    std::fs::File::open(png)
                        .expect("Can not open the input file."));
    
        let mut png_info = 
            input_png
                .read_info()
                .expect("Can not read information of png.");
    
        let mut buf = 
            vec![0; png_info.output_buffer_size()];
    
        let frame_info = 
            png_info
                .next_frame(&mut buf)
                .expect("Can not read frame from png.");
    
        if frame_info.bit_depth != png::BitDepth::Eight {
            panic!("PNG Frame must in 8 bits.");
        }
    
        (frame_info, buf)
    }
}

