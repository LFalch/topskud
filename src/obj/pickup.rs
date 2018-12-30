use crate::{
    util::Point2,
    io::tex::{Assets, Sprite},
};
use ggez::{
    GameResult, Context,
    graphics::self,
};

use std::fmt::{self, Debug};

use super::health::Health;

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
    pub fn apply(&self, health: &mut Health) {
        (self.pickup_type.ability)(health)
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        self.pickup_type.draw(self.pos, ctx, assets)
    }
}

#[derive(Copy, Clone)]
pub struct PickupType {
    pub spr: Sprite,
    ability: fn(&mut Health),
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PickupType")
            .field("spr", &self.spr)
            .finish()
    }
}

pub const PICKUPS: [PickupType; 3] = [
    PickupType {
        spr: Sprite::HealthPack,
        ability: health_pack
    },
    PickupType {
        spr: Sprite::Armour,
        ability: armour
    },
    PickupType {
        spr: Sprite::Adrenaline,
        ability: adrenaline,
    }
];
fn health_pack(health: &mut Health) {
    health.hp = 100.;
}
fn armour(health: &mut Health) {
    health.armour = 100.;
}
fn adrenaline(health: &mut Health) {
    health.hp += 100.;
}