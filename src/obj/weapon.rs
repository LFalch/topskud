use std::num::NonZeroU16;
use std::fmt::{self, Display};

use crate::{
    util::Point2,
    game::DELTA,
    io::{
        snd::{Sound, MediaPlayer},
        tex::Sprite,
    },
};
use ggez::{Context, GameResult};

use super::{Object, bullet::Bullet};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Weapon {
    pub name: &'static str,
    pub clip_size: NonZeroU16,
    pub damage: f32,
    /// Fraction of armour damage redirverted to hp damage
    pub penetration: f32,
    /// Time between each shot
    pub fire_rate: f32,
    /// Time to reload a new clip/magazine
    pub reload_time: f32,
    pub fire_mode: FireMode,
    pub shot_snd: Sound,
    pub cock_snd: Sound,
    pub reload_snd: Sound,
    pub impact_snd: Sound,
    pub entity_sprite: Sprite,
}

macro_rules! nzu16 {
    (0) => {
        unimplemented!()
    };
    ($n:expr) => {
        unsafe{NonZeroU16::new_unchecked($n)}
    }
}

pub const GLOCK: Weapon = Weapon {
    name: "Glack",
    clip_size: nzu16!(15),
    damage: 34.,
    penetration: 0.24,
    fire_rate: 0.25,
    reload_time: 1.6,
    fire_mode: FireMode::SemiAutomatic,
    shot_snd: Sound::Shot2,
    cock_snd: Sound::Cock,
    reload_snd: Sound::Reload,
    impact_snd: Sound::Impact,
    entity_sprite: Sprite::Glock,
};

pub const FIVE_SEVEN: Weapon = Weapon {
    name: "5-SeveN",
    clip_size: nzu16!(20),
    damage: 41.,
    penetration: 0.46,
    fire_rate: 0.20,
    reload_time: 1.3,
    fire_mode: FireMode::SemiAutomatic,
    shot_snd: Sound::Shot1,
    cock_snd: Sound::Cock,
    reload_snd: Sound::Reload,
    impact_snd: Sound::Impact,
    entity_sprite: Sprite::FiveSeven,
};

pub const M4A1: Weapon = Weapon {
    name: "M4A1",
    clip_size: nzu16!(30),
    damage: 52.,
    penetration: 0.51,
    fire_rate: 0.075,
    reload_time: 2.8,
    fire_mode: FireMode::Automatic,
    shot_snd: Sound::Shot1,
    cock_snd: Sound::Cock,
    reload_snd: Sound::Reload,
    impact_snd: Sound::Impact,
    entity_sprite: Sprite::M4,
};
pub const AK47: Weapon = Weapon {
    name: "AK-47",
    clip_size: nzu16!(30),
    damage: 65.,
    penetration: 0.22,
    fire_rate: 0.09,
    reload_time: 2.6,
    fire_mode: FireMode::Automatic,
    shot_snd: Sound::Shot1,
    cock_snd: Sound::Cock,
    reload_snd: Sound::Reload,
    impact_snd: Sound::Impact,
    entity_sprite: Sprite::Ak47,
};

impl Weapon {
    pub fn make_instance(&self) -> WeaponInstance<'_> {
        let cur_clip = self.clip_size.get();
        WeaponInstance {
            weapon: self,
            cur_clip,
            loading_time: 0.,
            ammo: cur_clip*3,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}/{}", self.weapon.name, self.cur_clip, self.ammo)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WeaponInstance<'a> {
    pub cur_clip: u16,
    pub ammo: u16,
    pub loading_time: f32,
    pub weapon: &'a Weapon,
}

impl Display for WeaponInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}/{}", self.weapon.name, self.cur_clip, self.ammo)
    }
}

impl<'a> WeaponInstance<'a> {
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
            cur_clip,
            ammo,
            weapon,
        }
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if self.loading_time <= DELTA {
            self.loading_time = 0.;
        } else {
            self.loading_time -= DELTA;
            if self.loading_time <= DELTA {
                mplayer.play(ctx, self.weapon.cock_snd)?;
            }
        }
        Ok(())
    }
    pub fn reload(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        let clip_size = self.weapon.clip_size.get();
        if self.cur_clip == clip_size {
            return Ok(())
        }

        self.loading_time = self.weapon.reload_time;

        let ammo_to_reload = self.weapon.clip_size.get() - self.cur_clip;

        if self.ammo < ammo_to_reload {
            self.cur_clip += self.ammo;
            self.ammo = 0;
        } else {
            self.ammo -= self.weapon.clip_size.get() - self.cur_clip;
            self.cur_clip = clip_size;
        }
        mplayer.play(ctx, self.weapon.reload_snd)
    }
    pub fn shoot(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<BulletMaker<'a>>> {
        if self.cur_clip > 0 && self.loading_time == 0. {
            self.cur_clip -= 1;
            if self.cur_clip > 0 {
                self.loading_time = self.weapon.fire_rate;
            }
            mplayer.play(ctx, self.weapon.shot_snd)?;
            Ok(Some(BulletMaker(self.weapon)))
        } else {
            Ok(None)
        }
    }
}

pub struct BulletMaker<'a>(&'a Weapon);
impl<'a> BulletMaker<'a> {
    pub fn make(self, obj: Object) -> Bullet<'a> {
        Bullet {
            obj,
            weapon: self.0
        }
    }
}
