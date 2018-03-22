use ::*;
use super::world::*;
use obj::enemy::Chaser;
use ggez::graphics::{Drawable, DrawMode, WHITE, Rect};
use ggez::graphics::spritebatch::SpriteBatch;

use rand::{thread_rng, Rng};

use game::Menu;

#[derive(Debug, Copy, Clone)]
enum Blood {
    B1,
    B2,
    B3,
}

struct BloodSplatter {
    ty: Blood,
    o: Object,
}

impl BloodSplatter {
    fn new(mut o: Object) -> Self {
        o.pos += 16. * angle_to_vec(o.rot);
        BloodSplatter {
            o,
            ty: *thread_rng().choose(&[
                Blood::B1,
                Blood::B2,
                Blood::B2,
                Blood::B3,
                Blood::B3,
            ]).unwrap(),
        }
    }
    fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        let spr = match self.ty {
            Blood::B1 => Sprite::Blood1,
            Blood::B2 => Sprite::Blood2,
            Blood::B3 => Sprite::Blood3,
        };
        self.o.draw(ctx, a.get_img(spr))
    }
}

/// The state of the game
pub struct Play {
    health: u8,
    world: World,
    holes: SpriteBatch,
    bloods: Vec<BloodSplatter>,
}

impl Play {
    pub fn new(level: Level, a: &Assets) -> Self {
        Play {
            health: 10,
            bloods: Vec::new(),
            world: World {
                enemies: level.enemies,
                bullets: Vec::new(),
                player: Object::new(level.start_point.unwrap_or(Point2::new(500., 500.))),
                grid: level.grid,
            },
            holes: SpriteBatch::new(a.get_img(Sprite::Hole).clone()),
        }
    }
}

impl GameState for Play {
    fn update(&mut self, s: &mut State) {
        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.pos += 500. * DELTA * angle_to_vec(bullet.rot);
            if bullet.is_on_solid(&self.world.grid) {
                self.holes.add(bullet.drawparams());
                deads.push(i);
            } else if (bullet.pos-self.world.player.pos).norm() <= 16. {
                deads.push(i);
                self.bloods.push(BloodSplatter::new(bullet.clone()));
                self.health = self.health.saturating_sub(1);
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
        }


        // Define player velocity here already because enemies need it
        let player_vel = Vector2::new(s.input.hor(), s.input.ver());

        let mut deads = Vec::new();
        for (e, enemy) in self.world.enemies.iter_mut().enumerate().rev() {
            if enemy.can_see(self.world.player.pos, &self.world.grid).0 {
                enemy.behaviour = Chaser::LastKnown{
                    pos: self.world.player.pos,
                    vel: player_vel,
                };

                if enemy.shoot == 0 {
                    let pos = enemy.obj.pos + 20. * angle_to_vec(enemy.obj.rot);
                    let mut bul = Object::new(pos);
                    bul.rot = enemy.obj.rot;

                    self.world.bullets.push(bul);

                    enemy.shoot = 10;
                } else {
                    enemy.shoot -= 1;
                }
            } else {
                enemy.shoot = 10;
            }
            enemy.update();
            let mut dead = None;
            for (i, bullet) in self.world.bullets.iter().enumerate().rev() {
                let dist = bullet.pos - enemy.obj.pos;
                if dist.norm() < 16. {
                    dead = Some(i);
                    enemy.health -= 1;

                    if !enemy.behaviour.chasing() {
                        enemy.behaviour = Chaser::LookAround{
                            dir: dist
                        };
                    }

                    self.bloods.push(BloodSplatter::new(bullet.clone()));
                    if enemy.health == 0 {
                        deads.push(e);
                    }
                    break
                }
            }
            if let Some(i) = dead {
                self.world.bullets.remove(i);
            }
        }
        for i in deads {
            self.world.enemies.remove(i);
        }

        let speed = if s.modifiers.shift {
            200.
        } else {
            100.
        };
        self.world.player.move_on_grid(player_vel, speed, &self.world.grid);
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) {
        let dist = s.mouse - s.offset - self.world.player.pos;

        if self.health == 0 {
            let e = Box::new(Menu::new(ctx, &s.assets, "", None).unwrap());
            s.switch(e);
        }
        self.world.player.rot = angle_from_vec(&dist);

        // Center the camera on the player
        let p = self.world.player.pos;
        s.focus_on(p);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, WHITE)?;
        self.world.grid.draw(ctx, &s.assets)?;

        for blood in &self.bloods {
            blood.draw(ctx, &s.assets)?;
        }

        graphics::set_color(ctx, graphics::BLACK)?;
        self.world.player.draw(ctx, s.assets.get_img(Sprite::Person))?;

        for enemy in &self.world.enemies {
            graphics::set_color(ctx, BLUE)?;
            enemy.draw_visibility_cone(ctx, 400.)?;

            let (can_see, ray_end) = enemy.can_see(self.world.player.pos, &self.world.grid);
            if can_see {
                graphics::set_color(ctx, GREEN)?;
                graphics::line(ctx, &[enemy.obj.pos, self.world.player.pos], 1.)?;
            } else if let Some(ray_end) = ray_end {
                graphics::set_color(ctx, RED)?;
                graphics::line(ctx, &[enemy.obj.pos, ray_end], 3.)?;
            }
            enemy.draw(ctx, &s.assets)?;
        }
        graphics::set_color(ctx, WHITE)?;
        for bullet in &self.world.bullets {
            bullet.draw(ctx, s.assets.get_img(Sprite::Bullet))?;
        }
        self.holes.draw_ex(ctx, Default::default())?;

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::BLACK)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 1., y: 1., w: 102., h: 26.})?;
        graphics::set_color(ctx, GREEN)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 2., y: 2., w: self.health as f32 * 10., h: 24.})?;

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
