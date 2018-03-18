use ::*;
use super::world::*;
use ggez::graphics::{Color, WHITE};

/// The state of the game
pub struct Play {
    world: World,
    running: bool,
}

impl Play {
    pub fn new(level: Level) -> Self {
        Play {
            running: false,
            world: World {
                bullets: Vec::new(),
                player: Object::new(Point2::new(500., 500.)),
                level,
            }
        }
    }
}


impl GameState for Play {
    fn update(&mut self, s: &mut State) {
        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.pos += 500. * DELTA * angle_to_vec(bullet.rot);
            if bullet.is_on_solid(&self.world.level) {
                deads.push(i);
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
        }

        let speed = if self.running {
            300.
        } else {
            175.
        };
        self.world.player.move_on_level(Vector2::new(s.input.hor(), s.input.ver()), speed, &self.world.level);
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
        graphics::set_color(ctx, WHITE)?;
        for bullet in &self.world.bullets {
            bullet.draw(ctx, s.assets.get_img(Sprite::Bullet))?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn key_down(&mut self, _s: &mut State, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            LShift => self.running = true,
            _ => (),
        }
    }
    fn key_up(&mut self, _s: &mut State, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            LShift => self.running = false,
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, btn: MouseButton) {
        if let MouseButton::Left = btn {
            let pos = self.world.player.pos + 16. * angle_to_vec(self.world.player.rot);
            let mut bul = Object::new(pos);
            bul.rot = self.world.player.rot;

            self.world.bullets.push(bul);
        }
    }
}
