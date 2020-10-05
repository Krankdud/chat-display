use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use sdl2::ttf::Sdl2TtfContext;
use std::path::Path;

pub struct TextRenderer<'a, T> {
    font: Font<'a, 'a>,
    texture_creator: &'a TextureCreator<T>,

    text_width: i32,
}

impl<'a, T> TextRenderer<'a, T> {
    pub fn new(
        ttf_ctx: &'a Sdl2TtfContext,
        texture_creator: &'a TextureCreator<T>,
        font: &str,
        font_size: u16,
        window_width: u32,
    ) -> Self {
        let font = ttf_ctx
            .load_font(Path::new(font), font_size)
            .expect("Could not load font");
        TextRenderer {
            font: font,
            texture_creator: texture_creator,
            text_width: (window_width - 4) as i32,
        }
    }

    fn draw_text(&self, canvas: &mut WindowCanvas, string: &str, x: i32, y: i32, color: Color) {
        if string.len() == 0 {
            return;
        }

        let text = self
            .font
            .render(string)
            .solid(color)
            .expect("Could not render text");
        let texture = text.as_texture(self.texture_creator).unwrap();
        let mut rect = text.rect();
        rect.set_x(x);
        rect.set_y(y);
        canvas.copy(&texture, None, rect).unwrap();
    }

    pub fn render(&self, canvas: &mut WindowCanvas, string: &str, x: i32, y: i32) {
        let (width, _height) = self.font.size_of(string).unwrap();
        if x + (width as i32) > self.text_width {
            let splits: Vec<&str> = string.split(' ').collect();
            let mut line = String::new();
            let mut len: u32 = 0;
            let mut y = y;
            let (space_width, _) = self.font.size_of(" ").unwrap();

            // Line wrapping
            for word in splits.iter() {
                let (sw, _sh) = self.font.size_of(word).unwrap();
                if x + ((len + sw) as i32) > self.text_width {
                    self.draw_text(canvas, &line, x, y, Color::WHITE);
                    y += self.font.height() - 16;
                    line.clear();
                    len = 0;
                } else {
                    line.push_str(word);
                    line.push_str(" ");
                    len += sw + space_width;
                }
            }
        } else {
            self.draw_text(canvas, string, x, y, Color::WHITE);
        }
    }
}