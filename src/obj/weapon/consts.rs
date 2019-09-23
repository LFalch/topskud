use super::{FireMode, Weapon};

use lazy_static::lazy_static;

use std::fs::File;
use std::io::Read;
use std::num::NonZeroU16;
use std::collections::HashMap;
use std::f32::consts::PI;

lazy_static!{
    pub static ref WEAPONS: HashMap<String, Weapon> = {
        let mut file = File::open("resources/weapons/specs.toml").expect("specs.toml file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Reading to succeed");

        let templates: HashMap<String, WeaponTemplate> = toml::from_str(&file_contents).expect("well-defined weapons");
        templates.into_iter().map(|(k, v)| (k, v.build())).collect()
    };
}

#[derive(Serialize, Deserialize)]
pub struct WeaponTemplate {
    name: Box<str>,
    clip_size: NonZeroU16,
    clips: NonZeroU16,
    damage: f32,
    penetration: f32,
    fire_rate: f32,
    reload_time: f32,
    fire_mode: FireMode,
    shot_snd: Box<str>,
    cock_snd: Option<Box<str>>,
    reload_snd: Option<Box<str>>,
    click_snd: Box<str>,
    impact_snd: Option<Box<str>>,
    entity_sprite: Box<str>,
    spray_pattern: Vec<f32>,
    spray_decay: f32,
    spray_repeat: usize,
}

const DEG2RAD: f32 = PI / 180.;

impl WeaponTemplate {
    fn build(self) -> Weapon {
        let WeaponTemplate {
            name,
            clip_size,
            clips,
            damage,
            penetration,
            fire_rate,
            reload_time,
            fire_mode,
            shot_snd,
            cock_snd,
            reload_snd,
            click_snd,
            impact_snd,
            entity_sprite,
            spray_pattern,
            spray_decay,
            spray_repeat,
        } = self;

        Weapon {
            name,
            clip_size,
            clips,
            damage,
            penetration,
            fire_rate,
            reload_time,
            fire_mode,
            shot_snd,
            cock_snd: cock_snd.unwrap_or_else(|| "cock".into()),
            reload_snd: reload_snd.unwrap_or_else(|| "reload".into()),
            click_snd,
            impact_snd: impact_snd.unwrap_or_else(|| "impact".into()),
            hands_sprite: (entity_sprite.to_string() + "_hands").into(),
            entity_sprite,
            spray_pattern: spray_pattern.into_iter().map(|deg| deg * DEG2RAD).collect(),
            spray_decay,
            spray_repeat,
        }
    }
}
