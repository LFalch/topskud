use ggez::graphics::{Color, Canvas};

use crate::{
    util::{Vector2},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::tex::{Assets, }
};
use super::{Object, player::Player, enemy::Enemy, health::Health, weapon::Weapon};

#[derive(Debug, Clone)]
pub struct Bullet<'a> {
    pub obj: Object,
    pub weapon: &'a Weapon,
    pub vel: Vector2,
}

impl Bullet<'_> {
    pub fn apply_damage(&self, health: &mut Health) {
        let dmg = self.weapon.damage * self.vel.norm() / self.weapon.bullet_speed;

        health.weapon_damage(dmg, self.weapon.penetration);
    }
    #[inline]
    pub fn draw(&self, canvas: &mut Canvas, a: &Assets) {
        let img = a.get_img(self.weapon.get_bullet_spr());
        self.obj.draw(canvas, &*img, Color::WHITE)
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_pos = self.vel * DELTA;

        const VELOCITY_DECREASE: f32 = 220. * DELTA;

        if self.vel.norm() <= VELOCITY_DECREASE {
            return Hit::Wall
        }
        
        // Check if we've hit a player or an enemy
        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            self.apply_damage(&mut player.health);
            return Hit::Player;
        }
        for (i, enem) in enemies.iter_mut().enumerate() {
            if Grid::dist_line_circle(start, d_pos, enem.pl.obj.pos) <= 16. {
                self.apply_damage(&mut enem.pl.health);
                return Hit::Enemy(i);
            }
        }

        // Decrease velocity after damage could've been dealt
        self.vel -= self.vel.normalize() * VELOCITY_DECREASE;

        // Ray cast bullet to see if we've hit a wall and move bullet accordingly
        let cast = grid.ray_cast(palette, start, d_pos, true);
        self.obj.pos = cast.into_point();
        if cast.full() {
            Hit::None
        } else {
            Hit::Wall
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Hit {
    Wall,
    Player,
    Enemy(usize),
    None,
}