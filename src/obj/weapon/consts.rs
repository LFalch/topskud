use std::num::NonZeroU16;

use crate::io::{
    snd::Sound,
    tex::Sprite,
};
use super::{FireMode, Weapon};

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
    clips: nzu16!(6),
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
    clips: nzu16!(5),
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
    clips: nzu16!(3),
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
    clips: nzu16!(3),
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
