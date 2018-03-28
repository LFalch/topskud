use ::*;
use ggez::nalgebra as na;
use ggez::graphics::DrawMode;
use io::qtree::QuadTree;

/// The state of the game
pub struct Qt {
    qt: QuadTree<Point2>,
    r_pos: Point2,
    r_rad: f32,
    hit_points: Vec<Point2>,
}

impl Qt {
    pub fn new(_ctx: &mut Context, s: &mut State) -> GameResult<Box<GameState>> {
        Ok(Box::new(Qt {
            qt: QuadTree::new(s.width as f32, s.height as f32, 2),
            r_pos: Point2::new(100., 200.),
            r_rad: 32.,
            hit_points: Vec::new(),
        }))
    }
}

impl GameState for Qt {
    fn logic(&mut self, _s: &mut State, _ctx: &mut Context) -> GameResult<()> {
        self.hit_points = self.qt.query_circular(self.r_pos, self.r_rad).into_iter().cloned().collect();
        Ok(())
    }
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.qt.draw(ctx)?;
        graphics::set_color(ctx, graphics::Color{r: 1., g: 1., b:0.,a: 1.})?;
        graphics::circle(ctx, DrawMode::Line(1.), self.r_pos, self.r_rad, 0.5)?;

        graphics::set_color(ctx, RED)?;
        graphics::points(ctx, &self.hit_points, 2.5)
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use MouseButton::*;
        match btn {
            Left => {
                self.qt.insert(s.mouse);
            }
            Right => {
                self.r_pos = s.mouse;
            }
            Middle => {
                self.r_rad = na::distance(&self.r_pos, &s.mouse);
            }
            _ => ()
        }
    }
}
