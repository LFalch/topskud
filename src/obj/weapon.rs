use std::num::NonZeroU16;
use std::fmt::{self, Display};

use crate::{
    util::{Point2, Rotation2},
    game::DELTA,
    io::{
        snd::MediaPlayer,
        tex::{PosText, Assets},
    },
};
use ggez::{Context, GameResult};

use super::{Object, bullet::Bullet};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FireMode {
    Automatic,
    SemiAutomatic,
    BoltAction,
    PumpAction{
        shell_load: u8,
    }
}

impl FireMode {
    #[inline]
    pub fn is_auto(self) -> bool {
        if let FireMode::Automatic = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub name: Box<str>,
    pub clip_size: NonZeroU16,
    pub clips: NonZeroU16,
    pub damage: f32,
    /// Fraction of armour damage rediverted to hp damage
    pub penetration: f32,
    /// Time between each shot
    pub fire_rate: f32,
    /// Time to reload a new clip/magazine
    pub reload_time: f32,
    pub fire_mode: FireMode,
    pub shot_snd: Box<str>,
    pub cock_snd: Box<str>,
    pub reload_snd: Box<str>,
    pub click_snd: Box<str>,
    pub impact_snd: Box<str>,
    pub entity_sprite: Box<str>,
    pub hands_sprite: Box<str>,
    pub spray_pattern: Box<[f32]>,
    pub spray_decay: f32,
    pub spray_repeat: usize,
}

mod consts;
pub use self::consts::*;

impl Weapon {
    pub fn make_instance(&self) -> WeaponInstance<'_> {
        let cur_clip = self.clip_size.get();
        WeaponInstance {
            weapon: self,
            cur_clip,
            loading_time: 0.,
            jerk: 0.,
            jerk_decay: 0.,
            spray_index: 0,
            ammo: cur_clip*self.clips.get(),
        }
    }
    pub fn make_drop(&self, pos: Point2) -> WeaponDrop<'_> {
        let cur_clip = self.clip_size.get();
        WeaponDrop {
            pos,
            cur_clip,
            ammo: cur_clip*self.clips.get(),
            weapon: self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeaponDrop<'a> {
    pub pos: Point2,
    pub cur_clip: u16,
    pub ammo: u16,
    pub weapon: &'a Weapon,
}

impl Display for WeaponDrop<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}/{}", self.weapon.name, self.cur_clip, self.ammo)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WeaponInstance<'a> {
    pub cur_clip: u16,
    pub ammo: u16,
    pub loading_time: f32,
    pub jerk: f32,
    pub jerk_decay: f32,
    pub spray_index: usize,
    pub weapon: &'a Weapon,
}

impl Display for WeaponInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}/{}", self.weapon.name, self.cur_clip, self.ammo)
    }
}

impl<'a> WeaponInstance<'a> {
    pub fn weapon_text(p: Point2, a: &Assets) -> PosText {
        a.text(p).and_text("BFG").and_text(" ").and_text("0").and_text("/").and_text("0").and_text(" (").and_text("0").and_text(" ").and_text("0").and_text("s)")
    }
    pub fn update_text(&self, text: &mut PosText) -> GameResult<()> {
        text
            .update(0, &*self.weapon.name)?
            .update(2, format!("{}", self.cur_clip))?
            .update(4, format!("{}", self.ammo))?
            .update(6, format!("{:.3}", self.jerk))?
            .update(8, format!("{:.1}", self.jerk_decay))?;
        Ok(())
    } 
    pub fn into_drop(self, pos: Point2) -> WeaponDrop<'a> {
        let WeaponInstance{cur_clip, ammo, weapon, ..} = self;
        WeaponDrop {
            pos,
            cur_clip,
            ammo,
            weapon,
        }
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_drop(wd: WeaponDrop<'a>) -> Self {
        let WeaponDrop{cur_clip, ammo, weapon, ..} = wd;
        Self {
            loading_time: 0.,
            jerk: 0.,
            jerk_decay: 0.,
            spray_index: 0,
            cur_clip,
            ammo,
            weapon,
        }
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if self.jerk_decay <= DELTA {
            self.jerk = 0.;
            self.jerk_decay = 0.;
            self.spray_index = 0;
        } else {
            self.jerk_decay -= DELTA;
        }
        if self.loading_time <= DELTA {
            self.loading_time = 0.;
        } else {
            self.loading_time -= DELTA;
            if self.loading_time <= DELTA {
                mplayer.play(ctx, &self.weapon.cock_snd)?;
            }
        }
        Ok(())
    }
    pub fn reload(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        let clip_size = self.weapon.clip_size.get();
        if self.cur_clip == clip_size || self.ammo == 0 {
            return Ok(())
        }

        self.loading_time = self.weapon.reload_time;

        let ammo_to_reload = self.weapon.clip_size.get() - self.cur_clip;

        if self.ammo < ammo_to_reload {
            self.cur_clip += self.ammo;
            self.ammo = 0;
        } else {
            self.ammo -= ammo_to_reload;
            self.cur_clip = clip_size;
        }
        mplayer.play(ctx, &self.weapon.reload_snd)
    }
    pub fn shoot(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<BulletMaker<'a>>> {
        if self.cur_clip > 0 && self.loading_time == 0. {
            self.cur_clip -= 1;
            if self.cur_clip > 0 {
                self.loading_time = self.weapon.fire_rate;
            }

            let jerk = self.jerk;

            self.jerk_decay = self.weapon.spray_decay;
            self.jerk += self.weapon.spray_pattern[self.spray_index];
            self.spray_index += 1; 
            if self.spray_index >= self.weapon.spray_pattern.len() {
                self.spray_index -= self.weapon.spray_repeat;
            }

            mplayer.play(ctx, &self.weapon.shot_snd)?;
            Ok(Some(BulletMaker(self.weapon, jerk)))
        } else {
            if self.cur_clip == 0 {
                mplayer.play(ctx, &self.weapon.click_snd)?;
            }
            Ok(None)
        }
    }
}

pub struct BulletMaker<'a>(&'a Weapon, f32);
impl<'a> BulletMaker<'a> {
    pub fn make(self, mut obj: Object, target: Point2) -> Bullet<'a> {
        let dest = target - obj.pos;

        obj.rot += self.1;
        Bullet {
            target: obj.pos + Rotation2::new(self.1) * dest,
            obj,
            weapon: self.0
        }
    }
}
