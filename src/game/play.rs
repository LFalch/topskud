use crate::*;
use super::world::*;
use crate::io::snd::Sound;
use crate::obj::enemy::Chaser;
use ggez::graphics::{Drawable, DrawMode, WHITE, Rect};
use ggez::graphics::spritebatch::SpriteBatch;

use rand::{thread_rng, prelude::SliceRandom};

use crate::game::StateSwitch;

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
            ty: *[
                Blood::B1,
                Blood::B2,
                Blood::B2,
                Blood::B3,
                Blood::B3,
            ].choose(&mut thread_rng()).unwrap(),
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
    victory_time: f32,
    misses: usize,
}

impl Play {
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<GameState>> {
        let level = if let Some(lvl) = s.level.clone() {
            lvl
        } else {
            let lvl = s.content.load_level()?;
            s.level = Some(lvl.clone());
            lvl
        };

        s.mplayer.play(ctx, Sound::Cock)?;

        Ok(Box::new(
            Play {
                misses: 0,
                victory_time: 0.,
                health: 10,
                bloods: Vec::new(),
                world: World {
                    enemies: level.enemies,
                    bullets: Vec::new(),
                    player: Object::new(level.start_point.unwrap_or(Point2::new(500., 500.))),
                    grid: level.grid,
                    exit: level.exit,
                    intels: level.intels,
                },
                holes: SpriteBatch::new(s.assets.get_img(Sprite::Hole).clone()),
            }
        ))
    }
}

impl GameState for Play {
    fn update(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.pos += 500. * DELTA * angle_to_vec(bullet.rot);
            if bullet.is_on_solid(&self.world.grid) {
                s.mplayer.play(ctx, Sound::Impact)?;
                self.holes.add(bullet.drawparams());
                self.misses += 1;
                deads.push(i);
            } else if (bullet.pos-self.world.player.pos).norm() <= 16. {
                deads.push(i);
                self.bloods.push(BloodSplatter::new(bullet.clone()));
                self.health = self.health.saturating_sub(1);
                s.mplayer.play(ctx, Sound::Hit)?;

                if self.health == 0 {
                    s.switch(StateSwitch::Lose(Statistics{
                        hits: self.bloods.len(),
                        misses: self.misses,
                        enemies_left: self.world.enemies.len(),
                        health_left: self.health,
                    }));
                    s.mplayer.play(ctx, Sound::Death)?;
                } else {
                    s.mplayer.play(ctx, Sound::Hurt)?;
                }
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
        }

        let mut deads = Vec::new();
        for (i, &intel) in self.world.intels.iter().enumerate().rev() {
            if (intel-self.world.player.pos).norm() <= 15. {
                deads.push(i);
                s.mplayer.play(ctx, Sound::Hit)?;
            }
        }
        for i in deads {
            self.world.intels.remove(i);
        }

        // Define player velocity here already because enemies need it
        let player_vel = Vector2::new(s.input.hor(), s.input.ver());

        let mut deads = Vec::new();
        for (e, enemy) in self.world.enemies.iter_mut().enumerate().rev() {
            if enemy.can_see(self.world.player.pos, &self.world.grid) {
                enemy.behaviour = Chaser::LastKnown{
                    pos: self.world.player.pos,
                    vel: player_vel,
                };

                if enemy.shoot == 0 {
                    let pos = enemy.obj.pos + 20. * angle_to_vec(enemy.obj.rot);
                    let mut bul = Object::new(pos);
                    bul.rot = enemy.obj.rot;

                    s.mplayer.play(ctx, Sound::Shot1)?;
                    self.world.bullets.push(bul);

                    enemy.shoot = 30;
                } else {
                    enemy.shoot -= 1;
                }
            } else {
                enemy.shoot = 30;
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
                    s.mplayer.play(ctx, Sound::Hit)?;

                    self.bloods.push(BloodSplatter::new(bullet.clone()));
                    if enemy.health == 0 {
                        s.mplayer.play(ctx, Sound::Death)?;
                        deads.push(e);
                    } else {
                        s.mplayer.play(ctx, Sound::Hurt)?;
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

        let game_won = match self.world.exit {
            Some(p) => self.world.intels.is_empty() && (p - self.world.player.pos).norm() < 32.,
            None => self.world.enemies.is_empty(),
        };

        if game_won && self.victory_time <= 0. {
            s.mplayer.play(ctx, Sound::Victory)?;
            self.victory_time += DELTA;
        } else if self.victory_time > 0. {
            self.victory_time += DELTA;
        }
        if self.victory_time >= 2. {
            s.level = None;
            s.switch(StateSwitch::Win(Statistics{
                hits: self.bloods.len(),
                misses: self.misses,
                enemies_left: self.world.enemies.len(),
                health_left: self.health,
            }));
        }
        Ok(())
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) -> GameResult<()> {
        let dist = s.mouse - s.offset - self.world.player.pos;

        self.world.player.rot = angle_from_vec(&dist);

        // Center the camera on the player
        let p = self.world.player.pos;
        s.focus_on(p);
        Ok(())
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, WHITE)?;
        self.world.grid.draw(ctx, &s.assets)?;

        self.holes.draw_ex(ctx, Default::default())?;

        for &intel in &self.world.intels {
            let drawparams = graphics::DrawParam {
                dest: intel,
                offset: Point2::new(0.5, 0.5),
                color: Some(graphics::WHITE),
                .. Default::default()
            };
            graphics::draw_ex(ctx, s.assets.get_img(Sprite::Intel), drawparams)?;
        }

        for blood in &self.bloods {
            blood.draw(ctx, &s.assets)?;
        }

        self.world.player.draw(ctx, s.assets.get_img(Sprite::Player))?;

        for enemy in &self.world.enemies {
            enemy.draw(ctx, &s.assets)?;
        }
        for bullet in &self.world.bullets {
            bullet.draw(ctx, s.assets.get_img(Sprite::Bullet))?;
        }

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
    fn mouse_up(&mut self, s: &mut State, ctx: &mut Context, btn: MouseButton) {
        if let MouseButton::Left = btn {
            let pos = self.world.player.pos + 16. * angle_to_vec(self.world.player.rot);
            let mut bul = Object::new(pos);
            bul.rot = self.world.player.rot;

            s.mplayer.play(ctx, Sound::Shot2).unwrap();
            self.world.bullets.push(bul);
        }
    }
}
