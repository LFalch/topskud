use std::num::NonZeroU16;

use super::{FireMode, Weapon};
use crate::io::{snd::Sound, tex::Sprite};

macro_rules! nzu16 {
    (0) => {
        unimplemented!()
    };
    ($n:expr) => {
        unsafe { NonZeroU16::new_unchecked($n) }
    };
}

macro_rules! weapons {
    (
        $(
            $name:ident {
                $($key:ident: $val:expr,)*
            };
        )*
    ) => {
        $(
            const $name: Weapon = Weapon {
                $($key: $val,)*
            };
        )*
        pub const WEAPONS: &'static [Weapon] = &[
            $(
                $name
            ),*
        ];
    };
}

use std::f32::consts::PI;
const DEG2RAD: f32 = PI / 180.;

macro_rules! spray {
    ($($val:expr),+) => {
        &[$(
            $val * DEG2RAD
        ),+]
    };
}

weapons! {
    // 0
    GLOCK {
        name: "Glack",
        clip_size: nzu16!(16),
        clips: nzu16!(7),
        damage: 34.,
        penetration: 0.24,
        fire_rate: 0.25,
        reload_time: 1.6,
        fire_mode: FireMode::SemiAutomatic,
        shot_snd: Sound::Shot2,
        cock_snd: Sound::Cock,
        click_snd: Sound::ClickPistol,
        reload_snd: Sound::Reload,
        impact_snd: Sound::Impact,
        entity_sprite: Sprite::Glock,
        hands_sprite: Sprite::GlockHands,
        spray_pattern: spray![6., -8., 4., -6., 2.5, 6., 4.],
        spray_decay: 0.43,
        spray_repeat: 2,
    };
    // 1
    FIVE_SEVEN {
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
        click_snd: Sound::ClickPistol,
        reload_snd: Sound::Reload,
        impact_snd: Sound::Impact,
        entity_sprite: Sprite::FiveSeven,
        hands_sprite: Sprite::FiveSevenHands,
        spray_pattern: spray![4., 6., -8., 4., -6., 4., -8., 6., 4.],
        spray_decay: 0.34,
        spray_repeat: 5,
    };
    // 2
    MAGNUM {
        name: "500-MG",
        clip_size: nzu16!(5),
        clips: nzu16!(4),
        damage: 111.,
        penetration: 0.05,
        fire_rate: 0.72,
        reload_time: 3.2,
        fire_mode: FireMode::SemiAutomatic,
        shot_snd: Sound::Shot1,
        cock_snd: Sound::Cock2,
        click_snd: Sound::ClickPistol,
        reload_snd: Sound::Reload,
        impact_snd: Sound::Impact,
        entity_sprite: Sprite::Magnum,
        hands_sprite: Sprite::MagnumHands,
        spray_pattern: spray![6., 2., -2.],
        spray_decay: 0.85,
        spray_repeat: 2,
    };
    // 3
    M4A1 {
        name: "M4A1",
        clip_size: nzu16!(30),
        clips: nzu16!(3),
        damage: 52.,
        penetration: 0.51,
        fire_rate: 0.075,
        reload_time: 2.8,
        fire_mode: FireMode::Automatic,
        shot_snd: Sound::Shot1,
        cock_snd: Sound::CockAk47,
        click_snd: Sound::ClickUzi,
        reload_snd: Sound::ReloadM4,
        impact_snd: Sound::Impact,
        entity_sprite: Sprite::M4,
        hands_sprite: Sprite::M4Hands,
        spray_pattern: spray![3.3, 4.2, -3., 3., -3., 2., -4., 3., 2.],
        spray_decay: 0.2,
        spray_repeat: 5,
    };
    // 4
    AK47 {
        name: "AK-47",
        clip_size: nzu16!(30),
        clips: nzu16!(3),
        damage: 65.,
        penetration: 0.22,
        fire_rate: 0.09,
        reload_time: 2.6,
        fire_mode: FireMode::Automatic,
        shot_snd: Sound::Shot1,
        cock_snd: Sound::CockAk47,
        click_snd: Sound::ClickUzi,
        reload_snd: Sound::Reload,
        impact_snd: Sound::Impact,
        entity_sprite: Sprite::Ak47,
        hands_sprite: Sprite::Ak47Hands,
        spray_pattern: spray![-3.3, -4.2, 3., -3., 3., -2., 4., -3., -2., 3.],
        spray_decay: 0.13,
        spray_repeat: 5,
    };
    // 5
    ARWP {
        name: "ARWP",
        clip_size: nzu16!(10),
        clips: nzu16!(4),
        damage: 130.,
        penetration: 0.8,
        fire_rate: 0.92,
        reload_time: 3.5,
        fire_mode: FireMode::BoltAction,
        shot_snd: Sound::Shot1,
        cock_snd: Sound::Cock2,
        click_snd: Sound::ClickPistol,
        reload_snd: Sound::ReloadM4,
        impact_snd: Sound::Impact,
        entity_sprite: Sprite::Arwp,
        hands_sprite: Sprite::ArwpHands,
        spray_pattern: spray![5.6, 1., -1.],
        spray_decay: 1.,
        spray_repeat: 2,
    };
}
