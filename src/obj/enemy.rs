use ggez::{
    graphics::{self, Point2, Vector2},
    nalgebra as na, Context, GameResult,
};

use crate::{
    game::{world::Grid, DELTA},
    io::{
        snd::MediaPlayer,
        tex::{Assets, Sprite},
    },
    util::{angle_from_vec, angle_to_vec},
};

use super::{player::Player, Object};

#[derive(Debug, Clone)]
pub enum Chaser {
    NoIntel,
    LastKnown { pos: Point2, vel: Vector2 },
    LookAround { dir: Vector2 },
}

impl Chaser {
    pub fn chasing(&self) -> bool {
        match *self {
            Chaser::LastKnown { .. } => true,
            _ => false,
        }
    }
}

impl Default for Chaser {
    fn default() -> Self {
        Chaser::NoIntel
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub pl: Player,
    #[serde(skip)]
    pub behaviour: Chaser,
}

pub const VISIBILITY: f32 = ::std::f32::consts::FRAC_PI_4;

impl Enemy {
    pub fn new(obj: Object) -> Enemy {
        Enemy {
            pl: Player::new(obj),
            behaviour: Chaser::NoIntel,
        }
    }
    pub fn draw_visibility_cone(&self, ctx: &mut Context, length: f32) -> GameResult<()> {
        let Object { pos, rot } = self.pl.obj;
        let dir1 = angle_to_vec(rot - VISIBILITY);
        let dir2 = angle_to_vec(rot + VISIBILITY);
        graphics::line(ctx, &[pos, pos + (length * dir1)], 1.5)?;
        graphics::line(ctx, &[pos, pos + (length * dir2)], 1.5)
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.pl.draw(ctx, a, Sprite::Enemy)
    }
    fn look_towards(&mut self, dist: Vector2) -> bool {
        let dir = angle_to_vec(self.pl.obj.rot);

        let rotation = na::angle(&dir, &dist);

        const ROTATION: f32 = 6. * DELTA;

        if rotation > ROTATION {
            if dir.perp(&dist) > 0. {
                self.pl.obj.rot += ROTATION;
            } else {
                self.pl.obj.rot -= ROTATION;
            }
            false
        } else {
            self.pl.obj.rot = angle_from_vec(dist);
            true
        }
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if let Some(wep) = &mut self.pl.wep {
            wep.update(ctx, mplayer)?;
            if wep.cur_clip == 0 && wep.loading_time == 0. {
                wep.reload(ctx, mplayer)?;
            }
        }
        match self.behaviour {
            Chaser::NoIntel => (),
            Chaser::LastKnown {
                pos: player_pos,
                vel,
            } => {
                let dist = player_pos - self.pl.obj.pos;
                self.look_towards(dist);

                let distance = dist.norm();
                const CHASE_SPEED: f32 = 100. * DELTA;

                if distance >= CHASE_SPEED {
                    let displace = CHASE_SPEED * dist / distance;
                    self.pl.obj.pos += displace;
                } else {
                    self.behaviour = Chaser::LookAround { dir: vel };
                }
            }
            Chaser::LookAround { dir } => {
                if self.look_towards(dir) {
                    self.behaviour = Chaser::NoIntel;
                }
            }
        }
        Ok(())
    }
    pub fn can_see(&self, p: Point2, grid: &Grid) -> bool {
        let dist = p - self.pl.obj.pos;
        let dir = angle_to_vec(self.pl.obj.rot);

        na::angle(&dir, &dist) <= VISIBILITY && grid.ray_cast(self.pl.obj.pos, dist, true).full()
    }
}
