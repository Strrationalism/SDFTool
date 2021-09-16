use std::{fs::File, io::Write, path::PathBuf};
use crate::mono_image::MonoImage;

struct AtlasRecord {
    character: char,
    page_id: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize
}

pub struct AtlasGenerator {
    page: MonoImage,

    page_id: usize,
    output_dir: PathBuf,
    
    x: usize,
    y: usize,
    current_height: usize,

    margin_x: usize,
    margin_y: usize,

    metadata: Vec<AtlasRecord>
}

impl AtlasGenerator {
    pub fn new(
        width: usize, 
        height: usize, 
        output_dir: PathBuf,
        margin_x: usize,
        margin_y: usize) 
        -> Self 
    {
        AtlasGenerator {
            page: MonoImage::new(width, height),
            page_id: 0,
            output_dir,
            x: 0,
            y: 0,
            current_height: 0,
            metadata: vec![],
            margin_x,
            margin_y
        }
    }

    fn next_line(&mut self) -> bool {
        if self.y + self.current_height < self.page.height {            
            self.y += self.current_height;
            self.current_height = 0;
            self.x = 0;
            true
        } else {
            false
        }
    }

    pub fn save_current_page(&self) {
        self.page.save_png(
            &self.output_dir.join(format!("{}.png", self.page_id)));
    }

    fn next_page(&mut self) {
        self.page_id += 1;
        self.current_height = 0;
        self.x = 0;
        self.y = 0;
        self.page.clear_color();
    }

    pub fn push(&mut self, ch: char, image: &MonoImage) {
        let width = image.width + 2 * self.margin_x;
        let height = image.height + 2 * self.margin_y;

        if width > self.page.width || height > self.page.height {
            panic!("Page size is too small!");
        }

        if self.x + width >= self.page.width {
            if !self.next_line() {
                self.save_current_page();
                self.next_page();
            }
        }

        self.current_height = height.max(self.current_height);

        for y in 0..image.height {
            for x in 0..image.width {
                let px = image.pixels[image.offset(x, y)];
                let target = 
                    self.page.offset(
                        self.x + self.margin_x + x, 
                        self.y + self.margin_y + y);

                self.page.pixels[target] = px;
            }
        }

        self.metadata.push(AtlasRecord {
            character: ch,
            page_id: self.page_id,
            x: self.x + self.margin_x,
            y: self.y + self.margin_y,
            w: image.width,
            h: image.height
        });

        self.x += width;
    }

    pub fn save_metadata(&self) {
        let csv_path = self.output_dir.join("metadata.csv");
        let mut out = File::create(csv_path).unwrap();

        out.write_all("char, page_id, x, y, width, height\n".as_bytes()).unwrap();

        for i in &self.metadata {
            let line = 
                format!(
                    "{}, {}, {}, {}, {}, {}\n", 
                    i.character as usize,
                    i.page_id,
                    i.x,
                    i.y,
                    i.w,
                    i.h);

            out.write_all(line.as_bytes()).unwrap();
        }
    }
}