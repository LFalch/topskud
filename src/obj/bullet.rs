use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::{Point2, angle_to_vec},
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
}

const SPEED: f32 = 1200.;
const HEADSHOT_BONUS: f32 = 1.5;

impl Bullet<'_> {
    pub fn apply_damage(&self, health: &mut Health, pos: Point2) -> bool {
        let headshot = (self.target - pos).norm() <= 12.;
        let dmg = if headshot {
            HEADSHOT_BONUS
        } else {
            1.
        } * self.weapon.damage;

        health.weapon_damage(dmg, self.weapon.penetration);
        headshot
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        let img = a.get_img(ctx, "common/bullet");
        self.obj.draw(ctx, &*img, WHITE)
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_pos = SPEED * DELTA * angle_to_vec(self.obj.rot);

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