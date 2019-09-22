use crate::{
    io::tex::{Assets, Sprite},
};
use ggez::{Context, GameResult, graphics::Color};

use super::Object;

#[derive(Debug, Copy, Clone)]
pub struct DecorationDecl {
    pub spr: Sprite,
    pub solid: bool,
}

const fn decl(spr: Sprite, solid: bool) -> DecorationDecl {
    DecorationDecl { spr, solid }
}

pub const DECORATIONS: &[DecorationDecl] = &[
    decl(Sprite::Chair1, false),
    decl(Sprite::Chair2, false),
    decl(Sprite::ChairBoss, false),
    decl(Sprite::LampPost, false),
    decl(Sprite::OfficePlant, false),
    decl(Sprite::OfficePlant2, false),
    decl(Sprite::OfficePlant3, false),
    decl(Sprite::Trashcan, true),
    decl(Sprite::ManholeCover, false),
    decl(Sprite::ManholeCover2, false),
    decl(Sprite::DeskLamp, false),
    decl(Sprite::WallLight, false),
    decl(Sprite::WallLight2, false),
    decl(Sprite::WallLight3, false),
    decl(Sprite::RoadMark, false),
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
        self.obj.draw(ctx, a.get_img(DECORATIONS[self.decl].spr), color)
    }
    #[inline]
    pub fn is_solid(&self) -> bool {
        DECORATIONS[self.decl].solid
    }
}