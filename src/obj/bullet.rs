use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::angle_to_vec,
    game::{
        DELTA,
        world::Grid,
    },
    io::tex::{Assets, }
};
use super::{Object, player::Player, enemy::Enemy, health::Health, weapon::Weapon};

#[derive(Debug, Clone)]
pub struct Bullet<'a> {
    pub obj: Object,
    pub weapon: &'a Weapon,
}

const SPEED: f32 = 1200.;

impl Bullet<'_> {
    #[inline]
    pub fn apply_damage(&self, health: &mut Health) {
        health.weapon_damage(self.weapon.damage, self.weapon.penetration)
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        let img = a.get_img(ctx, "common/bullet");
        self.obj.draw(ctx, &*img, WHITE)
    }
    pub fn update(&mut self, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_pos = SPEED * DELTA * angle_to_vec(self.obj.rot);

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
        let cast = grid.ray_cast(start, d_pos, true);
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