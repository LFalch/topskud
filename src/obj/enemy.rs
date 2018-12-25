use ggez::{
    Context, GameResult,
    nalgebra as na,
    graphics::{self, Point2, Vector2}
};

use crate::{
    angle_from_vec, angle_to_vec,
    io::tex::{Assets, Sprite},
    game::{DELTA, world::Grid},
};

use super::Object;

#[derive(Debug, Clone)]
pub enum Chaser {
    NoIntel,
    LastKnown{
        pos: Point2,
        vel: Vector2,
    },
    LookAround {
        dir: Vector2,
    }
}

impl Chaser {
    pub fn chasing(&self) -> bool {
        match *self {
            Chaser::LastKnown{..} => true,
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
    pub obj: Object,
    #[serde(skip)]
    pub behaviour: Chaser,
    #[serde(skip, default = "three")]
    pub health: u8,
    #[serde(skip)]
    pub shoot: u8,
}

fn three() -> u8 { 3 }

pub const VISIBILITY: f32 = ::std::f32::consts::FRAC_PI_4;

impl Enemy {
    pub fn new(obj: Object) -> Enemy {
        Enemy {
            shoot: 0,
            obj,
            health: 3,
            behaviour: Chaser::NoIntel,
        }
    }
    pub fn draw_visibility_cone(&self, ctx: &mut Context, length: f32) -> GameResult<()> {
        let dir1 = angle_to_vec(self.obj.rot - VISIBILITY);
        let dir2 = angle_to_vec(self.obj.rot + VISIBILITY);
        graphics::line(ctx, &[self.obj.pos, self.obj.pos + (length * dir1)], 1.5)?;
        graphics::line(ctx, &[self.obj.pos, self.obj.pos + (length * dir2)], 1.5)
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.obj.draw(ctx, a.get_img(Sprite::Enemy))
    }
    fn look_towards(&mut self, dist: Vector2) -> bool{
        let dir = angle_to_vec(self.obj.rot);

        let rotation = na::angle(&dir, &dist);

        const ROTATION: f32 = 6. * DELTA;

        if rotation > ROTATION {
            if dir.perp(&dist) > 0. {
                self.obj.rot += ROTATION;
            } else {
                self.obj.rot -= ROTATION;
            }
            false
        } else {
            self.obj.rot = angle_from_vec(&dist);
            true
        }
    }
    pub fn update(&mut self) {
        match self.behaviour {
            Chaser::NoIntel => (),
            Chaser::LastKnown{
                pos: player_pos,
                vel
            } => {
                let dist = player_pos-self.obj.pos;
                self.look_towards(dist);

                let distance = dist.norm();
                const CHASE_SPEED: f32 = 100. * DELTA;

                if distance >= CHASE_SPEED {
                    let displace = CHASE_SPEED * dist / distance;
                    self.obj.pos += displace;
                } else {
                    self.behaviour = Chaser::LookAround{dir: vel};
                }
            }
            Chaser::LookAround{dir} => {
                if self.look_towards(dir) {
                    self.behaviour = Chaser::NoIntel;
                }
            }
        }
    }
    pub fn can_see(&self, p: Point2, grid: &Grid) -> bool {
        let dist = p-self.obj.pos;
        let dir = angle_to_vec(self.obj.rot);

        if na::angle(&dir, &dist) <= VISIBILITY {
            let distance = dist.norm();

            let intervals = (distance / 16.).ceil() as u16;
            let ray = dist.normalize() * (distance / f32::from(intervals));

            let mut ray_end = self.obj.pos;

            for _ in 0..intervals {
                let (x, y) = Grid::snap(ray_end);
                if grid.is_solid(x, y) {
                    return false
                }

                ray_end += ray;
            }
            true
        } else {
            false
        }
    }
}
