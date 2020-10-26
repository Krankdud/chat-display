use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use sdl2::ttf::Sdl2TtfContext;
use std::collections::VecDeque;
use std::path::Path;

const LINE_OFFSET: i32 = 14;
const MAX_LINES: usize = 30;

struct Line {
    text: String,
    color: Color,
}

pub struct TextRenderer<'a, T> {
    font: Font<'a, 'a>,
    texture_creator: &'a TextureCreator<T>,

    text_width: i32,
    text_height: i32,
    margin: i32,

    lines: VecDeque<Line>,
}

impl<'a, T> TextRenderer<'a, T> {
    pub fn new(
        ttf_ctx: &'a Sdl2TtfContext,
        texture_creator: &'a TextureCreator<T>,
        font: &str,
        font_size: u16,
        window_width: u32,
        window_height: u32,
        margin: i32,
    ) -> Self {
        let font = ttf_ctx
            .load_font(Path::new(font), font_size)
            .expect("Could not load font");
        TextRenderer {
            font: font,
            texture_creator: texture_creator,
            text_width: (window_width as i32) - margin * 2,
            text_height: (window_height as i32) - margin * 2,
            margin: margin,
            lines: VecDeque::with_capacity(MAX_LINES),
        }
    }

    fn render_text(&self, canvas: &mut WindowCanvas, string: &str, x: i32, y: i32, color: Color) {
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

    fn render_line(&self, canvas: &mut WindowCanvas, data: &Line, x: i32, y: i32) -> i32 {
        let string = &data.text;
        let (width, _height) = self.font.size_of(string).unwrap();
        if x + (width as i32) > self.text_width {
            let splits: Vec<&str> = string.split(' ').collect();
            let mut lines: Vec<String> = vec![];
            let mut line = String::new();
            let mut len: u32 = 0;
            let (space_width, _) = self.font.size_of(" ").unwrap();

            // Line wrapping
            for word in splits.iter() {
                let (sw, _sh) = self.font.size_of(word).unwrap();
                if x + ((len + sw) as i32) > self.text_width {
                    lines.push(line.clone());
                    line.clear();
                    len = 0;
                }
                line.push_str(word);
                line.push_str(" ");
                len += sw + space_width;
            }

            // Push the leftover words
            if len > 0 {
                lines.push(line);
            }

            // Render lines
            let start_y = y - (self.font.height() - LINE_OFFSET) * ((lines.len() as i32) - 1);
            let mut y = start_y;
            for l in lines.iter() {
                self.render_text(canvas, &l, x, y, Color::WHITE);
                y += self.font.height() - LINE_OFFSET;
            }

            // Render colored name
            self.render_text(canvas, &splits[0], x, start_y, data.color);

            return start_y - self.font.height() + LINE_OFFSET;
        } else {
            self.render_text(canvas, string, x, y, Color::WHITE);

            // Render colored name
            let splits: Vec<&str> = string.split(' ').collect();
            self.render_text(canvas, &splits[0], x, y, data.color);
        }
        return y - self.font.height() + LINE_OFFSET;
    }

    pub fn render(&self, canvas: &mut WindowCanvas) {
        let x = self.margin;
        let mut y = self.margin + self.text_height - self.font.height() + LINE_OFFSET;
        for line in self.lines.iter() {
            y = self.render_line(canvas, line, x, y);
        }
    }

    pub fn push_line(&mut self, string: &str, color: &(u8, u8, u8)) {
        if self.lines.len() == MAX_LINES {
            self.lines.pop_back();
        }
        let line = Line {
            text: String::from(string),
            color: Color::RGB(color.0, color.1, color.2),
        };
        self.lines.push_front(line);
    }
}
