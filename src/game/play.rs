use ::*;
use super::world::*;

/// The state of the game
pub struct Play {
    world: World,
}

impl Play {
    pub fn new() -> Self {
        Play {
            world: World {
                player: Object::new(Point2::new(500., 500.)),
                // car: Car::new(100., 50., 375., 250.),
                level: Level::new(),
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

        // self.world.car.update(&self.input);
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) {
        let dist = s.mouse - s.offset - self.world.player.pos;

        self.world.player.rot = angle_from_vec(&dist);


        // Center the camera on the player
        let p = self.world.player.pos;
        s.focus_on(p);
    }

    // Draws everything
    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.world.level.draw(ctx, &s.assets)?;
        self.world.player.draw(ctx, s.assets.get_img(Sprite::Player))?;
        // self.world.car.obj.draw(ctx, self.assets.get_img(Sprite::Ferrari))?;
        // self.world.car.draw_lines(ctx)?;

        Ok(())
    }
    // Draws everything
    fn draw_hud(&mut self, _s: &State, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    /// Handle key down events
    fn key_down(&mut self, _s: &mut State, keycode: Keycode) {
        use Keycode::*;
        // Update input axes and quit game on Escape
        match keycode {
            S => (),
            _ => return,
        }
    }
    /// Handle key release events
    fn key_up(&mut self, _s: &mut State, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            X => save::load("save.lvl", &mut self.world.level).unwrap(),
            _ => return,
        }
    }
}
