use std::num::NonZeroU16;
use std::fmt::{self, Display};

use crate::obj::health::Health;

#[derive(Debug, Clone, Copy)]
pub enum FireMode {
    Automatic,
    SemiAutomatic,
    BoltAction,
    PumpAction{
        shell_load: u8,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Weapon {
    pub name: &'static str,
    pub clip_size: NonZeroU16,
    pub damage: f32,
    pub penetration: f32,
    pub fire_rate: f32,
    pub fire_mode: FireMode,
}

impl Weapon {
    pub fn apply_damage(&self, health: &mut Health) {
        health.weapon_damage(self.damage, self.penetration)
    }
}

macro_rules! nzu16 {
    (0) => {
        unimplemented!()
    };
    ($n:expr) => {
        unsafe{NonZeroU16::new_unchecked($n)}
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WeaponInstance<'a> {
    pub cur_clip: u16,
    pub clips: u16,
    pub loading_time: f32,
    pub weapon: &'a Weapon,
}

impl Display for WeaponInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}/{}", self.weapon.name, self.cur_clip, self.clips * self.weapon.clip_size.get())
    }
}

pub const GLOCK: Weapon = Weapon {
    name: "Glock",
    clip_size: nzu16!(7),
    damage: 36.,
    penetration: 0.4,
    fire_rate: 0.05,
    fire_mode: FireMode::SemiAutomatic,
};