use crate::{
    io::tex::{Assets, },
};
use ggez::{Context, GameResult, graphics::Color};

use super::Object;

#[derive(Debug, Copy, Clone)]
pub struct DecorationDecl {
    pub spr: &'static str,
    pub solid: bool,
}

const fn decl(spr: &'static str, solid: bool) -> DecorationDecl {
    DecorationDecl { spr, solid }
}

pub const DECORATIONS: &[DecorationDecl] = &[
    decl("decorations/chair1", false),
    decl("decorations/chair2", false),
    decl("decorations/chair_boss", false),
    decl("decorations/lamp_post", false),
    decl("decorations/office_plant", false),
    decl("decorations/office_plant2", false),
    decl("decorations/office_plant3", false),
    decl("decorations/trashcan", true),
    decl("decorations/manhole_cover", false),
    decl("decorations/manhole_cover2", false),
    decl("decorations/desk_lamp", false),
    decl("decorations/wall_light", false),
    decl("decorations/wall_light2", false),
    decl("decorations/wall_light3", false),
    decl("decorations/road_mark", false),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationObj {
    pub obj: Object,
    pub decl: usize,
}

impl DecorationObj {
    #[inline]
    pub fn new(obj: Object, decl: usize) -> Self {
        DecorationObj {
            obj, decl
        }
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets, color: Color) -> GameResult<()> {
        let img = a.get_img(ctx, DECORATIONS[self.decl].spr);
        self.obj.draw(ctx, &*img, color)
    }
    #[inline]
    pub fn is_solid(&self) -> bool {
        DECORATIONS[self.decl].solid
    }
}