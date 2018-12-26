use std::num::NonZeroU16;
use std::fmt::{self, Display};

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
    name: &'static str,
    clip_size: NonZeroU16,
    damage: f32,
    penetration: f32,
    fire_rate: f32,
    fire_mode: FireMode,
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
    cur_clip: u16,
    clips: u16,
    loading_time: f32,
    weapon: &'a Weapon,
}

impl Display for WeaponInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}/{}", self.weapon.name, self.cur_clip, self.clips * self.weapon.clip_size.get())
    }
}

#[allow(unused)]
const GLOCK: Weapon = Weapon {
    name: "Glock",
    clip_size: nzu16!(7),
    damage: 32.,
    penetration: 0.4,
    fire_rate: 0.05,
    fire_mode: FireMode::SemiAutomatic,
};