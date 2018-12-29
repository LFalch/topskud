use crate::{
    util::{
        BLUE, GREEN, RED,
        angle_to_vec, angle_from_vec,
        Vector2, Point2
    },
    io::{
        tex::{Assets, Sprite, PosText},
        snd::Sound,
    },
    obj::{Object, enemy::{Enemy, Chaser}, health::Health, weapon::{WeaponInstance, AK47}},
};
use ggez::{
    Context, GameResult,
    graphics::{
        self, Drawable, DrawMode, WHITE, Rect,
        spritebatch::SpriteBatch,
    },
    event::{Keycode, MouseButton}
};

use rand::{thread_rng, prelude::SliceRandom};
use super::{DELTA, State, GameState, StateSwitch, world::{Statistics, World}};

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
    hp_text: PosText,
    arm_text: PosText,
    reload_text: PosText,
    wep_text: PosText,
    status_text: PosText,
    health: Health,
    wep_inst: WeaponInstance<'static>,
    world: World,
    holes: SpriteBatch,
    bloods: Vec<BloodSplatter>,
    cur_pickup: Option<usize>,
    victory_time: f32,
    misses: usize,
}

impl Play {
    #[allow(clippy::new_ret_no_self)]
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
                hp_text: s.assets.text(ctx, Point2::new(4., 4.), "100")?,
                arm_text: s.assets.text(ctx, Point2::new(4., 33.), "100")?,
                reload_text: s.assets.text(ctx, Point2::new(4., 62.), "0.0s")?,
                wep_text: s.assets.text(ctx, Point2::new(2., 87.), "BFG 0/0")?,
                status_text: s.assets.text(ctx, Point2::new(s.width as f32 / 2., s.height as f32 / 2.+32.), "")?,
                misses: 0,
                victory_time: 0.,
                health: Health{hp: 100., armour: 100.},
                wep_inst: AK47.make_instance(),
                bloods: Vec::new(),
                cur_pickup: None,
                world: World {
                    enemies: level.enemies,
                    bullets: Vec::new(),
                    weapons: Vec::new(),
                    player: Object::new(level.start_point.unwrap_or_else(|| Point2::new(500., 500.))),
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
    #[allow(clippy::cyclomatic_complexity)]
    fn update(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        self.reload_text.update_text(&s.assets, ctx, &format!("{:.1}s", self.wep_inst.loading_time))?;
        self.wep_text.update_text(&s.assets, ctx, &format!("{}", self.wep_inst))?;
        if let Some(i) = self.cur_pickup {
            self.status_text.update_text(&s.assets, ctx, &format!("Press F to pick up {}", self.world.weapons[i]))?;
        } else {
            self.status_text.update_text(&s.assets, ctx, "")?;
        }

        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.obj.pos += 660. * DELTA * angle_to_vec(bullet.obj.rot);
            if bullet.obj.is_on_solid(&self.world.grid) {
                s.mplayer.play(ctx, bullet.weapon.impact_snd)?;
                self.holes.add(bullet.obj.drawparams());
                self.misses += 1;
                deads.push(i);
            } else if (bullet.obj.pos-self.world.player.pos).norm() <= 16. {
                deads.push(i);
                self.bloods.push(BloodSplatter::new(bullet.obj.clone()));
                bullet.apply_damage(&mut self.health);
                s.mplayer.play(ctx, Sound::Hit)?;

                self.hp_text.update_text(&s.assets, ctx, &format!("{:02.0}", self.health.hp))?;
                self.arm_text.update_text(&s.assets, ctx, &format!("{:02.0}", self.health.armour))?;

                if self.health.is_dead() {
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
        self.cur_pickup = None;
        for (i, weapon) in self.world.weapons.iter().enumerate().rev() {
            if (weapon.pos-self.world.player.pos).norm() <= 29. {
                self.cur_pickup = Some(i);
                break
            }
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

                if let Some(bm) = enemy.wep.shoot(ctx, &mut s.mplayer)? {
                    let pos = enemy.obj.pos + 20. * angle_to_vec(enemy.obj.rot);
                    let mut bul = Object::new(pos);
                    bul.rot = enemy.obj.rot;

                    self.world.bullets.push(bm.make(bul));
                }
            }
            enemy.update(ctx, &mut s.mplayer)?;
            let mut dead = None;
            for (i, bullet) in self.world.bullets.iter().enumerate().rev() {
                let dist = bullet.obj.pos - enemy.obj.pos;
                if dist.norm() < 16. {
                    dead = Some(i);
                    bullet.apply_damage(&mut enemy.health);

                    if !enemy.behaviour.chasing() {
                        enemy.behaviour = Chaser::LookAround{
                            dir: dist
                        };
                    }
                    s.mplayer.play(ctx, Sound::Hit)?;

                    self.bloods.push(BloodSplatter::new(bullet.obj.clone()));
                    if enemy.health.is_dead() {
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
            let Enemy{wep, obj: Object{pos, ..}, ..} = self.world.enemies.remove(i);
            self.world.weapons.push(wep.into_drop(pos));
        }

        let speed = if s.modifiers.shift {
            200.
        } else {
            100.
        };
        self.wep_inst.update(ctx, &mut s.mplayer)?;
        if s.mouse_down.left && self.wep_inst.weapon.fire_mode.is_auto() {
            if let Some(bm) = self.wep_inst.shoot(ctx, &mut s.mplayer)? {
                let pos = self.world.player.pos + 16. * angle_to_vec(self.world.player.rot);
                let mut bul = Object::new(pos);
                bul.rot = self.world.player.rot;

                self.world.bullets.push(bm.make(bul));
            }
        }
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

        self.world.player.rot = angle_from_vec(dist);

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
        for wep in &self.world.weapons {
            let drawparams = graphics::DrawParam {
                dest: wep.pos,
                offset: Point2::new(0.5, 0.5),
                color: Some(graphics::WHITE),
                .. Default::default()
            };
            graphics::draw_ex(ctx, s.assets.get_img(wep.weapon.entity_sprite), drawparams)?;
        }

        for blood in &self.bloods {
            blood.draw(ctx, &s.assets)?;
        }

        self.world.player.draw(ctx, s.assets.get_img(Sprite::Player))?;

        for enemy in &self.world.enemies {
            enemy.draw(ctx, &s.assets)?;
        }
        for bullet in &self.world.bullets {
            bullet.obj.draw(ctx, s.assets.get_img(Sprite::Bullet))?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        fn min(a: f32, b: f32) -> f32 {
            if a < b {
                a
            } else {
                b
            }
        }

        graphics::set_color(ctx, graphics::BLACK)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 1., y: 1., w: 102., h: 26.})?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 1., y: 29., w: 102., h: 26.})?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 1., y: 57., w: 102., h: 26.})?;
        graphics::set_color(ctx, GREEN)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 2., y: 2., w: self.health.hp, h: 24.})?;
        graphics::set_color(ctx, BLUE)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 2., y: 30., w: self.health.armour, h: 24.})?;
        graphics::set_color(ctx, RED)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 2., y: 58., w: min(1., self.wep_inst.loading_time)*100., h: 24.})?;
        graphics::set_color(ctx, WHITE)?;
        self.hp_text.draw_text(ctx)?;
        self.arm_text.draw_text(ctx)?;
        self.reload_text.draw_text(ctx)?;
        self.wep_text.draw_text(ctx)?;
        self.status_text.draw_center(ctx)?;

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
            if let Some(bm) = self.wep_inst.shoot(ctx, &mut s.mplayer).unwrap() {
                let pos = self.world.player.pos + 16. * angle_to_vec(self.world.player.rot);
                let mut bul = Object::new(pos);
                bul.rot = self.world.player.rot;

                self.world.bullets.push(bm.make(bul));
            }
        }
    }
    fn key_up(&mut self, s: &mut State, ctx: &mut Context, keycode: Keycode) {
        use self::Keycode::*;
        match keycode {
            R => self.wep_inst.reload(ctx, &mut s.mplayer).unwrap(),
            F => {
                if let Some(i) = self.cur_pickup {
                    let wep = self.wep_inst.into_drop(self.world.player.pos);
                    self.wep_inst = WeaponInstance::from_drop(std::mem::replace(&mut self.world.weapons[i], wep));
                }
            },
            _ => return,
        }
    }
}
