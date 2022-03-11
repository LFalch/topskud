use crate::{
    util::Point2,
    io::tex::{Assets, },
};
use ggez::{
    GameResult, Context,
    graphics,
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
    #[must_use]
    pub fn apply(&self, health: &mut Health) -> bool {
        (self.pickup_type.ability)(health)
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        self.pickup_type.draw(self.pos, ctx, assets)
    }
}

#[derive(Copy, Clone)]
pub struct PickupType {
    pub spr: &'static str,
    ability: fn(&mut Health) -> bool,
}

impl PickupType {
    #[inline]
    pub fn draw(&self, pos: Point2, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        let drawparams = graphics::DrawParam::default()
            .dest( pos)
            .offset( point!(0.5, 0.5));
        let img = assets.get_img(ctx, self.spr);
        graphics::draw(ctx, &*img, drawparams)
    }
}

impl Debug for PickupType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PickupType")
            .field("spr", &self.spr)
            .finish()
    }
}

pub const PICKUPS: [PickupType; 6] = [
    PickupType {
        spr: "pickups/health_pack",
        ability: health_pack
    },
    PickupType {
        spr: "pickups/armour",
        ability: armour
    },
    PickupType {
        spr: "pickups/adrenaline",
        ability: adrenaline,
    },
    PickupType {
        spr: "pickups/super_armour",
        ability: super_armour
    },
    PickupType {
        spr: "pickups/plaster",
        ability: plaster
    },
    PickupType {
        spr: "pickups/small_armour",
        ability: small_armour
    },
];
fn health_pack(health: &mut Health) -> bool {
    if health.hp >= 100. {
        false
    } else {
        health.hp = (health.hp + 75.).min(100.);
        true
    }
}
fn armour(health: &mut Health) -> bool {
    if health.armour >= 100. {
        false
    } else {
        health.armour = (health.armour + 75.).min(100.);
        true
    }
}
fn adrenaline(health: &mut Health) -> bool {
    if health.hp >= 200. {
        false
    } else {
        health.hp += 125.;
        true
    }
}
fn super_armour(health: &mut Health) -> bool {
    if health.armour >= 200. {
        false
    } else {
        health.armour += 125.;
        true
    }
}
fn plaster(health: &mut Health) -> bool {
    if health.hp >= 100. {
        false
    } else {
        health.hp = (health.hp + 10.).min(100.);
        true
    }
}
fn small_armour(health: &mut Health) -> bool {
    if health.armour >= 100. {
        false
    } else {
        health.armour = (health.armour + 10.).min(100.);
        true
    }
}