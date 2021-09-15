use clap::ArgMatches;
use rusttype::*;

use crate::mono_image::MonoImage;

pub struct BasicFontGenerator {
    font: Font<'static>,

    origin_scale: Scale,
    v_metrics: VMetrics,

    padding: (usize, usize)
}

impl BasicFontGenerator {
    pub fn generate(&self, c: char) -> MonoImage {
        let mut s = String::new();
        s.push(c);
        let (padding_x, padding_y) = self.padding;
        
        let glyph = 
            self.font.layout(
                &s, 
                self.origin_scale, 
                point(
                        padding_x as f32, 
                        padding_y  as f32+ self.v_metrics.ascent))
            .nth(0)
            .unwrap();

        let glyph_height = 
            (self.v_metrics.ascent - self.v_metrics.descent).ceil() as usize;

        let glyph_width =
            glyph.pixel_bounding_box().unwrap().max.x - glyph.pixel_bounding_box().unwrap().min.x;

        let mut image =
            MonoImage::new(
                glyph_width as usize + padding_x * 2, 
                glyph_height as usize + padding_y * 2);

        glyph.draw(|x, y, v|{
            image.set_pixel(
                x as usize + padding_x, 
                y as usize + padding_y, 
                if v >= 0.5 { 255 } else { 0 });
        });

        image
    }
}

impl From<&ArgMatches<'_>> for BasicFontGenerator {
    fn from(args: &ArgMatches) -> Self {
        let font_bytes =
            std::fs::read(args.value_of("INPUT").unwrap()).unwrap();

        let font = 
            Font::try_from_vec(font_bytes)
                .expect("Can not open font file.");

        let origin_scale = 
            Scale::uniform(
                args.value_of("origin-scale").unwrap().parse().unwrap());

        let v_metrics = font.v_metrics(origin_scale);

        let padding =
                (args.value_of("padding-x").unwrap().parse().unwrap(),
                 args.value_of("padding-y").unwrap().parse().unwrap());
        
        Self {
            font,
            origin_scale,
            v_metrics,
            padding
        }
    }
}