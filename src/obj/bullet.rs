use ggez::{Context, GameResult};

use crate::io::tex::{Assets, Sprite};
use super::{Object, health::Health, weapon::Weapon};

#[derive(Debug, Clone)]
pub struct Bullet<'a> {
    pub obj: Object,
    pub weapon: &'a Weapon,
}

impl Bullet<'_> {
    #[inline]
    pub fn apply_damage(&self, health: &mut Health) {
        health.weapon_damage(self.weapon.damage, self.weapon.penetration)
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.obj.draw(ctx, a.get_img(Sprite::Bullet))
    }
}