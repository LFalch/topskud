use std::collections::HashMap;
use std::cell::{RefCell, Ref};

use crate::util::{Point2, Vector2};

use ggez::context::Has;
use ggez::{Context, GameResult, GameError};
use ggez::graphics::{Canvas, Image, Text, TextFragment, Drawable, DrawParam, GraphicsContext, FontData};

/// All the assets
pub struct Assets {
    texes: RefCell<HashMap<String, Image>>,
}

const MISSING_TEXTURE: &str = "materials/missing";
const FONT_PATH: &str = "droidsansmono";

impl Assets {
    /// Initialises the assets with the context
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let font_data = FontData::from_path(ctx, "/common/DroidSansMono.ttf")?;
        ctx.gfx.add_font("droidsansmono", font_data);
        Ok(Assets {
            texes: RefCell::new(HashMap::with_capacity(64)),
        })
    }
    /// Gets the `Image` to draw from the sprite
    #[inline]
    pub fn get_img(&self, ctx: &mut Context, s: &str) -> Ref<Image> {
        if !self.texes.borrow().contains_key(s) {
            if let Ok(tex) = Image::from_path(ctx, &format!("/{}.png", s)) {
                self.texes.borrow_mut().insert(s.to_owned(), tex);
            } else if s != MISSING_TEXTURE {
                let img = self.get_img(ctx, MISSING_TEXTURE).clone();
                error!("Couldn't find texture {}. Loading default instead.", s);
                self.texes.borrow_mut().insert(s.to_owned(), img);
            } else {
                panic!("Missing texture not found");
            }
        }
        Ref::map(self.texes.borrow(), |ts| &ts[s])
    }
}

impl Assets {
    #[inline]
    pub fn raw_text(&self, size: f32) -> Text {
        let mut text = Text::default();
        text.set_font(FONT_PATH);
        text.set_scale(size);

        text
    }
    pub fn raw_text_with(&self, s: &str, size: f32) -> Text {
        let mut text = Text::new(s);
        text.set_font(FONT_PATH);
        text.set_scale(size);

        text
    }

    /// Make a positional text object
    #[inline]
    pub fn text(&self, pos: Point2) -> PosText {
        self.text_sized(pos, 18.)
    }
    /// Make a positional text object
    #[inline]
    pub fn text_sized(&self, pos: Point2, size: f32) -> PosText {
        PosText {
            pos,
            text: self.raw_text(size)
        }
    }
}

#[derive(Debug, Clone)]
/// A text with a position
///
/// Used for convenience so it's easier to update the text and rememeber their coordinates on the screen
pub struct PosText {
    pub pos: Point2,
    pub text: Text
}

impl PosText {
    pub fn and_text<T: Into<TextFragment>>(mut self, t: T) -> Self {
        self.text.add(t);
        self
    }
    /// Draw the text
    pub fn draw_text(&self, canvas: &mut Canvas) {
        self.text.draw(canvas, DrawParam::default().dest(self.pos))
    }
    pub fn draw_center(&self, canvas: &mut Canvas, gfx: &impl Has<GraphicsContext>) {
        let dims: Vector2 = self.text.measure(gfx).unwrap().into();
        let drawparams = DrawParam::new().dest(self.pos - 0.5 * dims);
        self.text.draw(canvas, drawparams)
    }
    pub fn update<T: Into<TextFragment>>(&mut self, fragment_index: usize, new_text: T) -> GameResult<&mut Self> {
        self.text.fragments_mut().get_mut(fragment_index).map(|t| *t = new_text.into()).ok_or_else(|| GameError::RenderError("Fragment did not exist".to_owned()))?;
        Ok(self)
    }
}
