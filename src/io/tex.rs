use std::collections::HashMap;

use ggez::{Context, GameResult};
use ggez::graphics::{Image, Font, Text, Point2, Drawable, DrawParam};

macro_rules! sprites {
    ($(
        $name:ident,
        $tex:expr,
        $width:expr,
        $height:expr,
    )*) => (
        #[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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
            texes: HashMap<Sprite, Image>,
            /// The font used for all the text
            pub font: Font,
            pub big_font: Font,
        }

        impl Assets {
            /// Initialises the assets with the context
            #[allow(clippy::new_ret_no_self)]
            pub fn new(ctx: &mut Context) -> GameResult<Self> {
                let mut texes = HashMap::new();
                $(
                    texes.insert(Sprite::$name, Image::new(ctx, concat!("/", $tex, ".png"))?);
                )*
                texes.shrink_to_fit();

                Ok(Assets {
                    texes,
                    font: Font::new(ctx, "/common/DroidSansMono.ttf", 14)?,
                    big_font: Font::new(ctx, "/common/DroidSansMono.ttf", 21)?,
                })
            }
            /// Gets the `Image` to draw from the sprite
            #[inline]
            pub fn get_img(&self, s: Sprite) -> &Image {
                &self.texes[&s]
            }
        }
    );
}

// Load all assets and specify their dimensions
sprites! {
    Player, "common/player", 32., 32.,
    Enemy, "common/enemy", 32., 32.,
    Crosshair, "common/crosshair", 32., 32.,
    Wall, "materials/wall", 32., 32.,
    Grass, "materials/grass", 32., 32.,
    Floor, "materials/floor", 32., 32.,
    Dirt, "materials/dirt", 32., 32.,
    WoodFloor, "materials/wood_floor", 32., 32.,
    Asphalt, "materials/asphalt", 32., 32.,
    Concrete, "materials/concrete", 32., 32.,
    Sand, "materials/sand", 32., 32.,
    Stairs, "materials/stairs", 32., 32.,
    Sidewalk, "materials/sidewalk", 32., 32.,
    WoodWall, "materials/wood_wall", 32., 32.,
    Missing, "materials/missing", 32., 32.,
    Bullet, "common/bullet", 16., 16.,
    Hole, "common/hole", 8., 8.,
    Blood1, "common/blood1", 32., 32.,
    Blood2, "common/blood2", 32., 32.,
    Blood3, "common/blood3", 32., 32.,
    Goal, "common/goal", 32., 32.,
    Intel, "common/intel", 32., 32.,
    HealthPack, "pickups/health_pack", 32., 32.,
    Armour, "pickups/armour", 32., 32.,
    Adrenaline, "pickups/adrenaline", 32., 32.,
    Trashcan, "decorations/trashcan", 32., 32.,
    LampPost, "decorations/lamp_post", 32., 32.,
    Chair1, "decorations/chair1", 32., 32.,
    Chair2, "decorations/chair2", 32., 32.,
    ChairBoss, "decorations/chair_boss", 32., 32.,
    OfficePlant, "decorations/officeplant", 32., 32.,
    OfficePlant2, "decorations/officeplant2", 32., 32.,
    OfficePlant3, "decorations/officeplant3", 32., 32.,
    ManholeCover, "decorations/manhole_cover", 32., 32.,
    ManholeCover2, "decorations/manhole_cover2", 32., 32.,
    DeskLamp, "decorations/desk_lamp", 32., 32.,
    WallLight, "decorations/wall_light", 32., 32.,
    WallLight2, "decorations/wall_light2", 32., 32.,
    WallLight3, "decorations/wall_light3", 32., 32.,
    RoadMark, "decorations/road_mark", 32., 32.,
    Machinery1, "decorations/machinery1", 32., 32.,
    Machinery2, "decorations/machinery2", 32., 32.,
    Machinery3, "decorations/machinery3", 32., 32.,
    Machinery4, "decorations/machinery4", 32., 32.,
    Glock, "weapons/glock", 32., 32.,
    GlockHands, "weapons/glock_hands", 32., 32.,
    FiveSeven, "weapons/five_seven", 32., 32.,
    FiveSevenHands, "weapons/five_seven_hands", 32., 32.,
    M4, "weapons/m4", 32., 32.,
    M4Hands, "weapons/m4_hands", 32., 32.,
    Ak47, "weapons/ak47", 32., 32.,
    Ak47Hands, "weapons/ak47_hands", 32., 32.,
    Magnum, "weapons/magnum", 32., 32.,
    MagnumHands, "weapons/magnum_hands", 32., 32.,
    Arwp, "weapons/arwp", 64., 32.,
    ArwpHands, "weapons/arwp_hands", 32., 32.,
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
