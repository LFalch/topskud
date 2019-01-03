use ggez::{Context, GameResult, graphics};

use crate::{
    util::{Point2, angle_to_vec},
    io::{
        snd::MediaPlayer,
        tex::{Assets, Sprite},
    },
};

use super::{Object, health::Health, weapon::WeaponInstance};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub obj: Object,
    #[serde(skip)]
    pub wep: Option<WeaponInstance<'static>>,
    #[serde(skip)]
    pub health: Health,
}

impl Player {
    #[inline]
    pub fn new(obj: Object) -> Self {
        Self {
            obj,
            wep: None,
            health: Health::default(),
        }
    }
    #[inline]
    pub fn from_point(p: Point2) -> Self {
        Player::new(Object::new(p))
    }
    #[inline]
    pub fn with_health(self, health: Health) -> Self {
        Self {
            health: health,
            .. self
        }
    }
    #[inline]
    pub fn with_weapon(self, wep: Option<WeaponInstance<'static>>) -> Self {
        Self {
            wep: wep,
            .. self
        }
    }

    /// Draw the object
    // pub fn draw(&self, ctx: &mut Context, img: &Image) -> GameResult<()> {
        // let drawparams = self.drawparams();
        // graphics::draw_ex(ctx, img, drawparams)
    // }

    #[inline]
    pub fn draw_player(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.draw(ctx, a, Sprite::Player)
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets, sprite: Sprite) -> GameResult<()> {
        if let Some(wep) = self.wep {
            let dp = graphics::DrawParam {
                dest: self.obj.pos+angle_to_vec(self.obj.rot)*16.,
                .. self.obj.drawparams()
            };

            graphics::draw_ex(ctx, a.get_img(wep.weapon.hands_sprite), dp)?;
        }
        self.obj.draw(ctx, a.get_img(sprite))
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if let Some(wep) = &mut self.wep {
            wep.update(ctx, mplayer)?;
        }
        Ok(())
    }
}
