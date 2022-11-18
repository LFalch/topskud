use crate::{
    util::Sstr,
    io::tex::{Assets, },
};
use ggez::{graphics::{Color, Canvas, GraphicsContext}, context::Has, GameResult};

use super::Object;

#[derive(Debug, Clone, Deserialize)]
pub struct OldDecoration {
    pub obj: Object,
    pub i: usize,
}

impl OldDecoration {
    pub fn renew(self) -> Decal {
        let OldDecoration{obj, i} = self;

        Decal {
            obj,
            spr: OLD_DECORATION_LIST[i]
        }
    }
}

const OLD_DECORATION_LIST: [&str; 15] = [
    "decorations/chair1",
    "decorations/chair2",
    "decorations/chair_boss",
    "decorations/lamp_post",
    "decorations/office_plant",
    "decorations/office_plant2",
    "decorations/office_plant3",
    "decorations/trashcan",
    "decorations/manhole_cover",
    "decorations/manhole_cover2",
    "decorations/desk_lamp",
    "decorations/wall_light",
    "decorations/wall_light2",
    "decorations/wall_light3",
    "decorations/road_mark"
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decal {
    pub obj: Object,
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    pub spr: Sstr
}

impl Decal {
    #[inline]
    pub fn new(obj: Object, spr: &'static str) -> Self {
        Decal {
            obj,
            spr,
        }
    }
    pub fn preload(&self, gfx: &impl Has<GraphicsContext>, a: &Assets) -> GameResult<()> {
        a.preload_imgs(gfx, [self.spr])
    }
    #[inline]
    pub fn draw(&self, canvas: &mut Canvas, a: &Assets, color: Color) {
        let img = a.get_img(&self.spr);
        self.obj.draw(canvas, &*img, color);
    }
}