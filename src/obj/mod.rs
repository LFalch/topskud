use crate::util::{Vector2, Point2};

use ggez::{Context, GameResult};
use ggez::graphics::{self, Image, Color};
// use ggez::nalgebra as na;

pub mod player;
pub mod enemy;
pub mod health;
pub mod weapon;
pub mod bullet;
pub mod pickup;
pub mod decal;
pub mod grenade;

use crate::game::world::{Grid, Palette};
use crate::game::DELTA;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A simple object that can be drawn to the screen
pub struct Object {
    #[serde(with = "crate::io::save::Point2Def")]
    /// The position of the object
    pub pos: Point2,
    /// The rotation of the obejct in radians
    pub rot: f32,
}

impl Object {
    /// Make a new physics object
    pub fn new(pos: Point2) -> Self {
        Object {
            pos,
            rot: 0.,
        }
    }
    pub fn with_rot(pos: Point2, rot: f32) -> Self {
        Object {
            pos,
            rot,
        }
    }
    #[inline]
    pub fn drawparams(&self) -> graphics::DrawParam {
        graphics::DrawParam {
            dest: self.pos.into(),
            rotation: self.rot,
            offset: Point2::new(0.5, 0.5).into(),
            .. Default::default()
        }
    }
    /// Draw the object
    pub fn draw(&self, ctx: &mut Context, img: &Image, color: Color) -> GameResult<()> {
        let drawparams = self.drawparams().color(color);
        graphics::draw(ctx, img, drawparams)
    }
    pub fn is_on_solid(&self, pal: &Palette, grid: &Grid) -> bool {
        let (x, y) = Grid::snap(self.pos);
        grid.is_solid(pal, x, y)
    }
    pub fn move_on_grid(&mut self, mut v: Vector2, speed: f32, pal: &Palette, grid: &Grid) {
        if v.x != 0. {
            let (xx, xy) = Grid::snap(self.pos + Vector2::new(16. * v.x, 0.));
            if grid.is_solid(pal, xx, xy) {
                v.x = 0.;
            }
        }
        if v.y != 0. {
            let (yx, yy) = Grid::snap(self.pos + Vector2::new(0., 16. * v.y));
            if grid.is_solid(pal, yx, yy) {
                v.y = 0.;
            }
        }

        if v.norm_squared() != 0. {
            v = v.normalize();
        }
        self.pos += v * speed * DELTA;
    }
}
