use std::collections::HashMap;
use std::cell::{RefCell, Ref};

use crate::util::{Point2, Vector2};

use ggez::{Context, GameResult, GameError};
use ggez::graphics::{Image, Font, Text, TextFragment, Drawable, DrawParam, Scale};

/// All the assets
pub struct Assets {
    texes: RefCell<HashMap<String, Image>>,
    /// The font used for all the text
    pub font: Font,
}

const MISSING_TEXTURE: &str = "materials/missing";

impl Assets {
    /// Initialises the assets with the context
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Assets {
            texes: RefCell::new(HashMap::with_capacity(64)),
            font: Font::new(ctx, "/common/DroidSansMono.ttf")?,
        })
    }
    /// Gets the `Image` to draw from the sprite
    #[inline]
    pub fn get_img(&self, ctx: &mut Context, s: &str) -> Ref<Image> {
        if !self.texes.borrow().contains_key(s) {
            if let Ok(tex) = Image::new(ctx, &format!("/{}.png", s)) {
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
        text.set_font(self.font, Scale::uniform(size));

        text
    }
    pub fn raw_text_with(&self, s: &str, size: f32) -> Text {
        let mut text = Text::new(s);
        text.set_font(self.font, Scale::uniform(size));

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
    pub fn draw_text(&self, ctx: &mut Context) -> GameResult<()> {
        self.text.draw(ctx, DrawParam {
            dest: self.pos.into(),
            .. Default::default()
        })
    }
    pub fn draw_center(&self, ctx: &mut Context) -> GameResult<()> {
        let (w, h) = self.text.dimensions(ctx);
        let drawparams = DrawParam::new().dest(self.pos - Vector2::new(w as f32 / 2., h as f32 / 2.));
        self.text.draw(ctx, drawparams)
    }
    pub fn update<T: Into<TextFragment>>(&mut self, fragment_index: usize, new_text: T) -> GameResult<&mut Self> {
        self.text.fragments_mut().get_mut(fragment_index).map(|t| *t = new_text.into()).ok_or_else(|| GameError::RenderError("Fragment did not exist".to_owned()))?;
        Ok(self)
    }
}
