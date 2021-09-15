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
    pub fn generate(&self, c: &str, buffer: &mut MonoImage) -> bool {
        assert!(c.chars().nth(0).is_some() && c.chars().nth(1).is_none());
        let (padding_x, padding_y) = self.padding;
        
        let glyph = 
            self.font.layout(
                c, 
                self.origin_scale, 
                point(0.0, self.v_metrics.ascent))
            .nth(0)
            .unwrap();

        let bounding_box = glyph.pixel_bounding_box();

        if bounding_box.is_none() { return false; }

        let bounding_box = bounding_box.unwrap();

        let glyph_height = 
            (self.v_metrics.ascent - self.v_metrics.descent).ceil() as usize;

        let glyph_width =
            bounding_box.max.x - bounding_box.min.x;

        buffer.clear_color();
        buffer.resize(
            glyph_width as usize + padding_x * 2, 
            glyph_height as usize + padding_y * 2);

        glyph.draw(|x, y, v|{
            buffer.set_pixel(
                (x as i32 + padding_x as i32) as usize, 
                (y as i32 + padding_y as i32 + bounding_box.min.y) as usize, 
                if v >= 0.5 { 255 } else { 0 });
        });

        true
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