use ggez::{Context, GameResult};

use crate::{
    util::{angle_to_vec, Vector2},
    game::{
        DELTA,
        world::Grid,
    },
    io::{
        snd::{Sound, MediaPlayer},
        tex::{Assets, Sprite},
    },
};
use super::{Object, player::Player, enemy::Enemy, health::Health};

#[derive(Debug, Default, Clone, Copy)]
pub struct Utilities {
    pub grenades: u8,
}

#[derive(Debug, Clone)]
pub struct Grenade {
    pub obj: Object,
    pub vel: Vector2,
}

const DEC: f32 = 0.5;

impl Grenade {
    #[inline]
    pub fn apply_damage(&self, health: &mut Health) {
        health.weapon_damage(55., 85.);
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        // TODO add sprite
        self.obj.draw(ctx, a.get_img(Sprite::ManholeCover))
    }
    pub fn update(&mut self, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Option<Explosion> {
        let start = self.obj.pos;
        let acc = DEC * -self.vel;
        let d_pos = 0.5 * DELTA * acc + self.vel * DELTA;
        let d_acc = acc * DELTA;
        if self.vel.norm() > d_acc.norm() {
            self.vel += d_acc;
        } else {
            return Some(Explosion);
        }

        let closest_p = Grid::closest_point_of_line_to_circle(start, d_pos, player.obj.pos);
        let r_player = player.obj.pos - closest_p;
        if r_player.norm() <= 16. {
            self.vel -= self.vel.dot(&r_player)/r_player.norm_squared() * r_player;

            self.obj.pos = closest_p + self.vel * (DELTA * (1. - ((closest_p - start).norm()/d_pos.norm())));
            return None;
        }
        for enem in enemies.iter_mut() {
            let closest_e = Grid::closest_point_of_line_to_circle(start, d_pos, enem.pl.obj.pos);
            let r_enemy = enem.pl.obj.pos - closest_e;
            if r_enemy.norm() <= 16. {
                self.vel -= self.vel.dot(&r_enemy)/r_enemy.norm_squared() * r_enemy;

                self.obj.pos = closest_e + self.vel * (DELTA * (1. - ((closest_p - start).norm()/d_pos.norm())));
                return None;
            }
        }
        let cast = grid.ray_cast(start, d_pos, true);
        self.obj.pos = cast.into_point();
        if cast.half() {
            self.vel = -self.vel;
            // TODO: Move it the amount it should've moved
            // TODO: Don't just turn it around
        }
        None
    }
}

impl Utilities {
    pub fn throw_grenade(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<GrenadeMaker>> {
        if self.grenades > 0 {
            self.grenades -= 1;

            mplayer.play(ctx, Sound::Impact)?;
            Ok(Some(GrenadeMaker(50.)))
        } else {
            mplayer.play(ctx, Sound::Cock)?;
            Ok(None)
        }
    }
}

pub struct GrenadeMaker(f32);
impl GrenadeMaker {
    pub fn make(self, mut obj: Object) -> Grenade {
        let vel = angle_to_vec(obj.rot) * self.0;
        obj.rot = 0.;
        Grenade {
            vel,
            obj,
        }
    }
}

// TODO: Add who was hit by the explosion
pub struct Explosion;
