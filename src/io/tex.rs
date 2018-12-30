use ggez::{Context, GameResult};
use ggez::graphics::{Image, Font, Text, Point2, Drawable, DrawParam};

macro_rules! sprites {
    ($(
        $name:ident,
        $tex:ident,
        $width:expr,
        $height:expr,
    )*) => (
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
        /// An object to reference a sprite in the `Assets`
        #[allow(missing_docs)]
        pub enum Sprite {
            $($name,)*
        }

        impl Sprite {
            /// Width of the sprite
            pub fn width(&self) -> f32 {
                match *self {
                    $(
                        Sprite::$name => $width,
                    )*
                }
            }
            /// Height of the sprite
            pub fn height(&self) -> f32 {
                match *self {
                    $(
                        Sprite::$name => $height,
                    )*
                }
            }
        }
        /// All the assets
        pub struct Assets {
            $(
                $tex: Image,
            )*
            /// The font used for all the text
            pub font: Font,
            pub big_font: Font,
        }

        impl Assets {
            /// Initialises the assets with the context
            #[allow(clippy::new_ret_no_self)]
            pub fn new(ctx: &mut Context) -> GameResult<Self> {
                $(
                    let $tex = Image::new(ctx, concat!("/", stringify!($tex), ".png"))?;
                )*

                Ok(Assets {
                    $(
                        $tex,
                    )*
                    font: Font::new(ctx, "/FiraMono.ttf", 14)?,
                    big_font: Font::new(ctx, "/FiraMono.ttf", 21)?,
                })
            }
            /// Gets the `Image` to draw from the sprite
            pub fn get_img(&self, s: Sprite) -> &Image {
                match s {
                    $(
                        Sprite::$name => &self.$tex,
                    )*
                }
            }
        }
    );
}

/// Load all assets and specify their dimensions
sprites! {
    Player, player, 32., 32.,
    Enemy, enemy, 32., 32.,
    Crosshair, crosshair, 32., 32.,
    Wall, wall, 32., 32.,
    Grass, grass, 32., 32.,
    Floor, floor, 32., 32.,
    Dirt, dirt, 32., 32.,
    WoodFloor, wood_floor, 32., 32.,
    Missing, missing, 32., 32.,
    Bullet, bullet, 16., 16.,
    Hole, hole, 8., 8.,
    Asphalt, asphalt, 32., 32.,
    Concrete, concrete, 32., 32.,
    Sand, sand, 32., 32.,
    Blood1, blood1, 32., 32.,
    Blood2, blood2, 32., 32.,
    Blood3, blood3, 32., 32.,
    Goal, goal, 32., 32.,
    Intel, intel, 32., 32.,
    HealthPack, health_pack, 32., 32.,
    Armour, armour, 32., 32.,
    Trashcan, trashcan, 32., 32.,
    Glock, glock, 32., 32.,
    FiveSeven, five_seven, 32., 32.,
    M4, m4, 32., 32.,
    Ak47, ak47, 32., 32.,
    Magnum, magnum, 32., 32.,
    Arwp, arwp, 64., 32.,
}

impl Assets {
    /// Make a positional text object
    pub fn text(&self, context: &mut Context, pos: Point2, text: &str) -> GameResult<PosText> {
        let text = Text::new(context, text, &self.font)?;
        Ok(PosText {
            pos,
            text
        })
    }
    /// Make a positional text object
    pub fn text_big(&self, context: &mut Context, pos: Point2, text: &str) -> GameResult<PosText> {
        let text = Text::new(context, text, &self.big_font)?;
        Ok(PosText {
            pos,
            text
        })
    }
}

#[derive(Debug, Clone)]
/// A text with a position
///
/// Used for convenience so it's easier to update the text and rememeber their coordinates on the screen
pub struct PosText {
    pub pos: Point2,
    text: Text
}

impl PosText {
    /// Draw the text
    pub fn draw_text(&self, ctx: &mut Context) -> GameResult<()> {
        self.text.draw(ctx, self.pos, 0.)
    }
    pub fn draw_center(&self, ctx: &mut Context) -> GameResult<()> {
        let drawparams = DrawParam {
            dest: self.pos,
            offset: Point2::new(0.5, 0.5),
            .. Default::default()
        };
        self.text.draw_ex(ctx, drawparams)
    }
    /// Update the text
    pub fn update_text(&mut self, a: &Assets, ctx: &mut Context, text: &str) -> GameResult<()> {
        if text != self.text.contents() {
            self.text = Text::new(ctx, text, &a.font)?;
        }
        Ok(())
    }
}
