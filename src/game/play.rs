use ::*;
use super::world::*;
use io::snd::Sound;
use obj::enemy::Chaser;
use ggez::graphics::{Drawable, DrawMode, WHITE, Rect};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::audio::Source;

use rand::{thread_rng, Rng};

use game::StateSwitch;

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

pub struct MultiSource {
    srcs: Box<[Source]>,
    idx: usize
}

impl MultiSource {
    fn new(srcs: Box<[Source]>) -> Self {
        MultiSource {
            srcs,
            idx: 0,
        }
    }
    fn play(&mut self) -> GameResult<()> {
        self.srcs[self.idx].play()?;
        self.idx += 1;
        if self.idx >= self.srcs.len() {
            self.idx = 0;
        }
        Ok(())
    }
}

/// The state of the game
pub struct Play {
    health: u8,
    world: World,
    holes: SpriteBatch,
    bloods: Vec<BloodSplatter>,
    shot: MultiSource,
    hit: Source,
    hurt: Source,
    impact: MultiSource,
    death: Source,
    victory: Source,
    victory_time: f32,
}

impl Drop for Play {
    fn drop(&mut self) {
        for s in &*self.shot.srcs {
            s.stop();
        }
        for s in &*self.impact.srcs {
            s.stop();
        }
        self.hit.stop();
        self.hurt.stop();
        self.death.stop();
        self.victory.stop();
    }
}

impl Play {
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<GameState>> {
        let s1 = s.sounds.make_source(ctx, Sound::Shot1)?;
        let s2 = s.sounds.make_source(ctx, Sound::Shot2)?;
        let hit = s.sounds.make_source(ctx, Sound::Hit)?;
        let hurt = s.sounds.make_source(ctx, Sound::Hurt)?;
        let impact = s.sounds.make_source(ctx, Sound::Impact)?;
        let impact1 = s.sounds.make_source(ctx, Sound::Impact)?;
        let impact2 = s.sounds.make_source(ctx, Sound::Impact)?;
        let death = s.sounds.make_source(ctx, Sound::Death)?;
        let victory = s.sounds.make_source(ctx, Sound::Victory)?;


        let level = if let Some(lvl) = s.level.clone() {
            lvl
        } else {
            let lvl = Level::load(&s.save)?;
            s.level = Some(lvl.clone());
            lvl
        };

        Ok(Box::new(
            Play {
                victory_time: 0.,
                health: 10,
                bloods: Vec::new(),
                world: World {
                    enemies: level.enemies,
                    bullets: Vec::new(),
                    player: Object::new(level.start_point.unwrap_or(Point2::new(500., 500.))),
                    grid: level.grid,
                    goal: level.goal,
                },
                shot: MultiSource::new(Box::new([s1, s2])),
                hit,
                hurt,
                impact: MultiSource::new(Box::new([impact, impact1, impact2])),
                victory,
                death,
                holes: SpriteBatch::new(s.assets.get_img(Sprite::Hole).clone()),
            }
        ))
    }
}

impl GameState for Play {
    fn update(&mut self, s: &mut State) -> GameResult<()> {
        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.pos += 500. * DELTA * angle_to_vec(bullet.rot);
            if bullet.is_on_solid(&self.world.grid) {
                self.impact.play()?;
                self.holes.add(bullet.drawparams());
                deads.push(i);
            } else if (bullet.pos-self.world.player.pos).norm() <= 16. {
                deads.push(i);
                self.bloods.push(BloodSplatter::new(bullet.clone()));
                self.health = self.health.saturating_sub(1);
                self.hit.play()?;

                if self.health == 0 {
                    self.hurt.stop();
                    self.death.stop();
                    self.hit.stop();
                    s.switch(StateSwitch::Menu);
                } else {
                    self.hurt.play()?;
                }
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
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

                    self.shot.play()?;
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
                    self.hit.play()?;

                    self.bloods.push(BloodSplatter::new(bullet.clone()));
                    if enemy.health == 0 {
                        self.hurt.play()?;
                        deads.push(e);
                    } else {
                        self.death.play()?;
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

        let game_won = match self.world.goal {
            Goal::Point(p) => (p - self.world.player.pos).norm() < 32.,
            Goal::KillAll => self.world.enemies.is_empty(),
        };

        if game_won && self.victory_time <= 0. {
            self.victory.play()?;
            self.victory_time += DELTA;
        } else if self.victory_time > 0. {
            self.victory_time += DELTA;
        }
        if self.victory_time >= 2. {
            s.switch(StateSwitch::Menu);
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

        for blood in &self.bloods {
            blood.draw(ctx, &s.assets)?;
        }

        graphics::set_color(ctx, graphics::BLACK)?;
        self.world.player.draw(ctx, s.assets.get_img(Sprite::Person))?;

        for enemy in &self.world.enemies {
            graphics::set_color(ctx, BLUE)?;

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

            self.shot.play().unwrap();
            self.world.bullets.push(bul);
        }
    }
}
