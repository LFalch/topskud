use ggez::{Context, GameResult};
use ggez::graphics::{self, Point2, Vector2};

use obj::Object;
use game::world::Grid;
use ::{angle_from_vec, angle_to_vec, Assets, Sprite, RED, DELTA};

use ggez::nalgebra as na;

#[derive(Debug, Clone)]
pub enum Chaser {
    NoIntel,
    LastKnown(Point2),
    LookAround {
        turn: f32,
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
        graphics::set_color(ctx, RED)?;
        self.obj.draw(ctx, a.get_img(Sprite::Person))
    }
    fn look_towards(&mut self, dist: Vector2) {
        let dir = angle_to_vec(self.obj.rot);

        let rotation = na::angle(&dir, &dist);

        const ROTATION: f32 = 3. * DELTA;

        if rotation > ROTATION {
            if dir.perp(&dist) > 0. {
                self.obj.rot += ROTATION;
            } else {
                self.obj.rot -= ROTATION;
            }
        } else {
            self.obj.rot = angle_from_vec(&dist);
        }
    }
    pub fn update(&mut self) {
        match self.behaviour {
            Chaser::NoIntel => (),
            Chaser::LastKnown(player_pos) => {
                let dist = player_pos-self.obj.pos;
                self.look_towards(dist);

                let distance = dist.norm();
                const CHASE_SPEED: f32 = 100. * DELTA;

                if distance >= CHASE_SPEED {
                    let displace = CHASE_SPEED * dist / distance;
                    self.obj.pos += displace;
                } else {
                    self.behaviour = Chaser::LookAround{turn: ::std::f32::consts::PI};
                }
            }
            Chaser::LookAround{turn} => {
                const ROTATION: f32 = ::std::f32::consts::FRAC_PI_6 * DELTA;

                if turn > ROTATION {
                    self.obj.rot += ROTATION;
                    self.behaviour = Chaser::LookAround{turn: turn - ROTATION};
                } else {
                    self.obj.rot += turn;
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
            let ray = dist.normalize() * (distance / intervals as f32);

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
