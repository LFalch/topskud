use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::{Vector2, Point2},
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
    pub target: Point2,
    pub vel: Vector2,
}

const HEADSHOT_BONUS: f32 = 1.5;

impl Bullet<'_> {
    pub fn apply_damage(&self, health: &mut Health, pos: Point2) -> bool {
        let headshot = (self.target - pos).norm() <= 12.;
        let dmg = if headshot {
            HEADSHOT_BONUS
        } else {
            1.
        } * self.weapon.damage * self.vel.norm() / self.weapon.bullet_speed;

        health.weapon_damage(dmg, self.weapon.penetration);
        headshot
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        let img = a.get_img(ctx, self.weapon.get_bullet_spr());
        self.obj.draw(ctx, &*img, WHITE)
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_pos = self.vel * DELTA;

        const VELOCITY_DECREASE: f32 = 220. * DELTA;

        if self.vel.norm() <= VELOCITY_DECREASE {
            return Hit::Wall
        }
        self.vel -= self.vel.normalize() * VELOCITY_DECREASE;

        // Check if we've hit a player or an enemy
        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            let hs = self.apply_damage(&mut player.health, player.obj.pos);
            return Hit::Player(hs);
        }
        for (i, enem) in enemies.iter_mut().enumerate() {
            if Grid::dist_line_circle(start, d_pos, enem.pl.obj.pos) <= 16. {
                let hs = self.apply_damage(&mut enem.pl.health, enem.pl.obj.pos);
                return Hit::Enemy(i, hs);
            }
        }

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
    Player(bool),
    Enemy(usize, bool),
    None,
}