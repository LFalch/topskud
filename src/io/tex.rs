use std::collections::HashMap;
use std::cell::{RefCell, Ref};

use crate::util::Point2;

use ggez::context::Has;
use ggez::{Context, GameResult, GameError};
use ggez::graphics::{Canvas, Image, Text, TextFragment, Drawable, DrawParam, GraphicsContext, FontData, TextLayout};

/// All the assets
pub struct Assets {
    missing_img: Image,
    texes: RefCell<HashMap<Box<str>, Image>>,
    load_queue: RefCell<Vec<Box<str>>>,
}

const FONT_PATH: &str = "droidsansmono";

impl Assets {
    /// Initialises the assets with the context
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let font_data = FontData::from_path(ctx, "/common/DroidSansMono.ttf")?;
        ctx.gfx.add_font("droidsansmono", font_data);
        Ok(Assets {
            missing_img: Image::from_bytes(ctx, include_bytes!("../../resources/materials/missing.png"))?,
            texes: RefCell::new(HashMap::with_capacity(64)),
            load_queue: RefCell::new(Vec::with_capacity(10)),
        })
    }
    /// Gets the `Image` to draw from the sprite
    /// 
    /// ## Note
    /// 
    /// The return value might be tentative, if you need the image to be useful use `get_or_load_img`
    #[inline]
    pub fn get_img(&self, s: &str) -> Ref<Image> {
        self.check_and_queue(s);
        Ref::map(self.texes.borrow(), |ts| &ts[s])
    }
    /// Gets the `Image` to draw from sprite name but makes sure it's fully loaded.
    /// This is neccesary if you're storing the Image someplace e.g. in an InstanceArray.
    pub fn get_or_load_img(&self, gfx: &impl Has<GraphicsContext>, s: &str) -> GameResult<Ref<Image>> {
        self.check_and_queue(s);

        // If the texture name is in the load queue, process the load queue now
        if self.load_queue.borrow().iter().any(|t| &**t == s) {
            self.inner_process_queue(gfx)?;
        }

        Ok(self.get_img(s))
    }
    pub fn preload_imgs<'a>(&'a self, gfx: &impl Has<GraphicsContext>, ss: impl 'a + IntoIterator<Item=&'a str>) -> GameResult<()> {
        for s in ss {
            self.check_and_queue(s);
        }

        // Just load the whole queue, simpler than finding out what to load
        self.inner_process_queue(gfx)?;
        Ok(())
    }

    #[inline(always)]
    pub fn process_queue(&mut self, gfx: &impl Has<GraphicsContext>) -> GameResult<()> {
        self.inner_process_queue(gfx)
    }

    /// Queues texture if unloaded
    fn check_and_queue(&self, s: &str) {
        if !self.texes.borrow().contains_key(s) {
            self.load_queue.borrow_mut().push(Box::from(s));
            self.texes.borrow_mut().insert(Box::from(s), self.missing_img.clone());
        }
    }

    fn inner_process_queue(&self, gfx: &impl Has<GraphicsContext>) -> GameResult<()> {
        #[cfg(debug_assertions)]
        if !self.load_queue.borrow().is_empty() {
            let queue = self.load_queue.borrow();
            debug!("Loading textures {:?}", queue);
        }

        for name in self.load_queue.borrow_mut().drain(..) {
            let img = match Image::from_path(gfx, &format!("/{name}.png")) {
                Ok(tex) => tex,
                Err(GameError::ResourceNotFound(_, _)) => {
                    error!("Couldn't find texture {}. Loading default instead.", name);
                    self.missing_img.clone()
                }
                Err(e) => return Err(e),
            };

            self.texes.borrow_mut().insert(name, img);
        }
        Ok(())
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
    pub fn centered(mut self) -> Self {
        self.text.set_layout(TextLayout::center());
        self
    }
    /// Draw the text
    pub fn draw_text(&self, canvas: &mut Canvas) {
        self.text.draw(canvas, DrawParam::default().dest(self.pos))
    }
    pub fn update<T: Into<TextFragment>>(&mut self, fragment_index: usize, new_text: T) -> GameResult<&mut Self> {
        self.text.fragments_mut().get_mut(fragment_index).map(|t| *t = new_text.into()).ok_or_else(|| GameError::RenderError("Fragment did not exist".to_owned()))?;
        Ok(self)
    }
}
