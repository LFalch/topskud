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
    pub wep: Option<WeaponInstance<'static>>,
    pub health: Health,
}

impl Player {
    pub fn new(p: Point2) -> Self {
        Self {
            obj: Object::new(p),
            wep: None,
            health: Health::default(),
        }
    }
    pub fn with_health(self, health: Health) -> Self {
        Self {
            health: health,
            .. self
        }
    }
    pub fn with_weapon(self, wep: Option<WeaponInstance<'static>>) -> Self {
        Self {
            wep: wep,
            .. self
        }
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.obj.draw(ctx, a.get_img(Sprite::Player))
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if let Some(wep) = &mut self.wep {
            wep.update(ctx, mplayer)?;
        }
        Ok(())
    }
}
