use ::*;
use super::world::*;
use ggez::graphics::{Color, WHITE};

/// The state of the game
pub struct Play {
    world: World,
}

impl Play {
    pub fn new(level: Level) -> Self {
        Play {
            world: World {
                player: Object::new(Point2::new(500., 500.)),
                level,
            }
        }
    }
}


impl GameState for Play {
    fn update(&mut self, s: &mut State) {
        let mut v = Vector2::new(s.input.hor(), s.input.ver());
        let (cx, cy) = Level::snap(self.world.player.pos + 16. * v);

        if !self.world.level.get(cx, cy).solid() {
            if v.norm_squared() != 0. {
                v = v.normalize();
            }
            self.world.player.pos += v * 175. * DELTA;
        }
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) {
        let dist = s.mouse - s.offset - self.world.player.pos;

        self.world.player.rot = angle_from_vec(&dist);

        // Center the camera on the player
        let p = self.world.player.pos;
        s.focus_on(p);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.world.level.draw(ctx, &s.assets)?;
        graphics::set_color(ctx, Color{r:0.,g:0.,b:0.,a:1.})?;
        self.world.player.draw(ctx, s.assets.get_img(Sprite::Person))?;

        Ok(())
    }
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, WHITE)
    }
}
