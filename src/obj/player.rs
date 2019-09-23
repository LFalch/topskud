use ggez::{Context, GameResult, graphics::{self, WHITE, Color}};

use crate::{
    util::{Point2, angle_to_vec},
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
};

use super::{Object, health::Health, weapon::WeaponInstance, grenade::Utilities};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub obj: Object,
    #[serde(skip)]
    pub wep: Option<WeaponInstance<'static>>,
    #[serde(skip)]
    pub health: Health,
    #[serde(skip)]
    pub utilities: Utilities,
}

impl Player {
    #[inline]
    pub fn new(obj: Object) -> Self {
        Self {
            obj,
            wep: None,
            health: Health::default(),
            utilities: Utilities::default(),
        }
    }
    #[inline]
    pub fn from_point(p: Point2) -> Self {
        Player::new(Object::new(p))
    }
    #[inline]
    pub fn with_health(self, health: Health) -> Self {
        Self {
            health,
            .. self
        }
    }
    #[inline]
    pub fn with_weapon(self, wep: Option<WeaponInstance<'static>>) -> Self {
        Self {
            wep,
            .. self
        }
    }

    #[inline]
    pub fn draw_player(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.draw(ctx, a, "common/player", WHITE)
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets, sprite: &str, color: Color) -> GameResult<()> {
        if let Some(wep) = self.wep {
            let dp = graphics::DrawParam {
                dest: (self.obj.pos+angle_to_vec(self.obj.rot)*16.).into(),
                color,
                .. self.obj.drawparams()
            };

            let img = a.get_img(ctx, &wep.weapon.hands_sprite);
            graphics::draw(ctx, &*img, dp)?;
        }
        let img = a.get_img(ctx, sprite);
        self.obj.draw(ctx, &*img, color)
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if let Some(wep) = &mut self.wep {
            wep.update(ctx, mplayer)?;
        }
        Ok(())
    }
}
