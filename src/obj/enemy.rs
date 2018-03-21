use ggez::{Context, GameResult};
use ggez::graphics::{self, Point2};

use obj::Object;
use game::world::Grid;
use ::{angle_to_vec, Assets, Sprite, RED};

use ggez::nalgebra as na;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub obj: Object,
}

pub const VISIBILITY: f32 = ::std::f32::consts::FRAC_PI_4;

impl Enemy {
    pub fn new(obj: Object) -> Enemy {
        Enemy {
            obj
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
