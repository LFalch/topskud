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
                enemies: level.enemies,
                bullets: Vec::new(),
                holes: Vec::new(),
                player: Object::new(level.start_point.unwrap_or(Point2::new(500., 500.))),
                grid: level.grid,
            }
        }
    }
}

impl GameState for Play {
    fn update(&mut self, s: &mut State) {
        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.pos += 500. * DELTA * angle_to_vec(bullet.rot);
            if bullet.is_on_solid(&self.world.grid) {
                self.world.holes.push(bullet.clone());
                deads.push(i);
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
        }

        let speed = if s.modifiers.shift {
            300.
        } else {
            175.
        };
        self.world.player.move_on_grid(Vector2::new(s.input.hor(), s.input.ver()), speed, &self.world.grid);
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) {
        let dist = s.mouse - s.offset - self.world.player.pos;

        self.world.player.rot = angle_from_vec(&dist);

        // Center the camera on the player
        let p = self.world.player.pos;
        s.focus_on(p);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, WHITE)?;
        self.world.grid.draw(ctx, &s.assets)?;
        graphics::set_color(ctx, Color{r:0.,g:0.,b:0.,a:1.})?;
        self.world.player.draw(ctx, s.assets.get_img(Sprite::Person))?;
        for enemy in &self.world.enemies {
            enemy.draw(ctx, &s.assets)?;
        }
        graphics::set_color(ctx, WHITE)?;
        for bullet in &self.world.bullets {
            bullet.draw(ctx, s.assets.get_img(Sprite::Bullet))?;
        }
        for hole in &self.world.holes {
            hole.draw(ctx, s.assets.get_img(Sprite::Hole))?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, RED)?;
        let drawparams = graphics::DrawParam {
            dest: s.mouse,
            offset: Point2::new(0.5, 0.5),
            .. Default::default()
        };
        graphics::draw_ex(ctx, s.assets.get_img(Sprite::Crosshair), drawparams)
    }
    fn mouse_up(&mut self, _s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        if let MouseButton::Left = btn {
            let pos = self.world.player.pos + 16. * angle_to_vec(self.world.player.rot);
            let mut bul = Object::new(pos);
            bul.rot = self.world.player.rot;

            self.world.bullets.push(bul);
        }
    }
}
