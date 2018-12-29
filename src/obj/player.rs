use ggez::{Context, GameResult};

use crate::{
    util::Point2,
    io::{
        snd::MediaPlayer,
        tex::{Assets, Sprite},
    },
};

use super::{Object, health::Health, weapon::WeaponInstance};

#[derive(Debug, Clone)]
pub struct Player {
    pub obj: Object,
    pub wep: WeaponInstance<'static>,
    pub health: Health,
}

impl Player {
    pub fn new(p: Point2, wep: WeaponInstance<'static>, health: Health) -> Self {
        Self {
            obj: Object::new(p),
            wep,
            health,
        }
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.obj.draw(ctx, a.get_img(Sprite::Player))
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        self.wep.update(ctx, mplayer)?;
        Ok(())
    }
}
