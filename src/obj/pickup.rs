use crate::{
    util::Point2,
    io::tex::{Assets, Sprite},
};
use ggez::{
    GameResult, Context,
    graphics::self,
};

use std::fmt::{self, Debug};

use super::player::Player;

#[derive(Debug, Clone)]
pub struct Pickup {
    pub pos: Point2,
    pub pickup_type: &'static PickupType
}

impl Pickup {
    #[inline]
    pub fn new(pos: Point2, i: u8) -> Self {
        Self {
            pos,
            pickup_type: &PICKUPS[i as usize]
        }
    }
    #[inline]
    pub fn apply(&self, player: &mut Player) {
        (self.pickup_type.ability)(self, player)
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        self.pickup_type.draw(self.pos, ctx, assets)
    }
}

#[derive(Copy, Clone)]
pub struct PickupType {
    pub spr: Sprite,
    ability: fn(&Pickup, &mut Player),
}

impl PickupType {
    #[inline]
    pub fn draw(&self, pos: Point2, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        let drawparams = graphics::DrawParam {
            dest: pos,
            offset: Point2::new(0.5, 0.5),
            .. Default::default()
        };
        graphics::draw_ex(ctx, assets.get_img(self.spr), drawparams)
    }
}

impl Debug for PickupType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PickupType")
            .field("spr", &self.spr)
            .finish()
    }
}

pub const PICKUPS: [PickupType; 2] = [
    PickupType {
        spr: Sprite::HealthPack,
        ability: health_pack
    },
    PickupType {
        spr: Sprite::Armour,
        ability: armour
    }
];
fn health_pack(_p: &Pickup, pl: &mut Player) {
    pl.health.hp = 100.;
}
fn armour(_p: &Pickup, pl: &mut Player) {
    pl.health.armour = 100.;
}