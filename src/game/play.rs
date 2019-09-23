use crate::{
    ext::FloatExt,
    util::{
        BLUE, GREEN, RED,
        angle_to_vec, angle_from_vec,
        ver, hor,
        Vector2, Point2
    },
    io::{
        tex::{Assets, PosText},
        snd::Sound,
    },
    obj::{Object, pickup::Pickup, player::Player, enemy::{Enemy, Chaser}, health::Health, weapon::WeaponInstance, grenade::Explosion},
};
use ggez::{
    Context, GameResult,
    graphics::{
        self, Drawable, DrawMode, Rect,
        MeshBuilder, Mesh, WHITE,
        spritebatch::SpriteBatch,
    },
    input::{
        keyboard::{self, KeyMods, KeyCode},
        mouse::{self, MouseButton}
    },
};

use rand::{thread_rng, prelude::SliceRandom};
use super::{DELTA, State, GameState, StateSwitch, world::{Level, Statistics, World}};

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
            Blood::B1 => "common/blood1",
            Blood::B2 => "common/blood2",
            Blood::B3 => "common/blood3",
        };
        let img = a.get_img(ctx, spr);
        self.o.draw(ctx, &*img, WHITE)
    }
}
/// The state of the game
pub struct Play {
    hp_text: PosText,
    arm_text: PosText,
    reload_text: PosText,
    wep_text: PosText,
    status_text: PosText,
    hud: Hud,
    world: World,
    holes: SpriteBatch,
    bloods: Vec<BloodSplatter>,
    cur_pickup: Option<usize>,
    victory_time: f32,
    misses: usize,
    initial: (Health, Option<WeaponInstance<'static>>),
    level: Level,
}

impl Play {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State, level: Level, pl: Option<(Health, Option<WeaponInstance<'static>>)>) -> GameResult<Box<dyn GameState>> {
        let mut player = Player::from_point(level.start_point.unwrap_or_else(|| Point2::new(500., 500.)));
        if let Some((h, w)) = pl {
            player = player.with_health(h).with_weapon(w);
        };

        Ok(Box::new(
            Play {
                level: level.clone(),
                initial: (player.health, player.wep),
                hp_text: s.assets.text(Point2::new(4., 4.)).and_text("100"),
                arm_text: s.assets.text(Point2::new(4., 33.)).and_text("100"),
                reload_text: s.assets.text(Point2::new(4., 62.)).and_text("0.0").and_text("s"),
                wep_text: WeaponInstance::weapon_text(Point2::new(2., 87.), &s.assets),
                status_text: s.assets.text(Point2::new(s.width as f32 / 2., s.height as f32 / 2. + 32.)).and_text(""),
                hud: Hud::new(ctx)?,
                misses: 0,
                victory_time: 0.,
                bloods: Vec::new(),
                cur_pickup: None,
                world: {
                    let mut world = World {
                        enemies: level.enemies,
                        bullets: Vec::new(),
                        grenades: Vec::new(),
                        weapons: level.weapons,
                        player,
                        grid: level.grid,
                        exit: level.exit,
                        intels: level.intels,
                        decorations: level.decorations,
                        pickups: level.pickups.into_iter().map(|(p, i)| Pickup::new(p, i)).collect(),
                    };
                    world.enemy_pickup();
                    world.player_pickup();

                    if world.player.wep.is_none() {
                        eprintln!("Warning: player has no weapon");
                    }

                    for enemy_pos in world.enemies.iter().filter_map(|enemy| if enemy.pl.wep.is_none() {Some(enemy.pl.obj.pos)}else{None}) {
                        eprintln!("Warning: enemy at {:.2} has no weapon", enemy_pos)
                    }

                    world
                },
                holes: SpriteBatch::new(s.assets.get_img(ctx, "common/hole").clone()),
            }
        ))
    }
}

impl GameState for Play {
    #[allow(clippy::cognitive_complexity)]
    fn update(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        self.hp_text.update(0, format!("{:02.0}", self.world.player.health.hp))?;
        self.arm_text.update(0, format!("{:02.0}", self.world.player.health.armour))?;
        if let Some(wep) = self.world.player.wep {
            self.reload_text.update(0, format!("{:.1}", wep.loading_time))?;
            wep.update_text(&mut self.wep_text)?;
        }
        if let Some(i) = self.cur_pickup {
            self.status_text.text.fragments_mut()[0]= format!("Press F to pick up {}", self.world.weapons[i]).into();
        } else {
            self.status_text.update(0, "")?;
        }

        let mut deads = Vec::new();
        for (i, grenade) in self.world.grenades.iter_mut().enumerate().rev() {
            let expl = grenade.update(&self.world.grid, &mut self.world.player, &mut *self.world.enemies);

            if let Some(Explosion{player_hit, enemy_hits}) = expl {
                deads.push(i);
                s.mplayer.play(ctx, Sound::Boom)?;

                if player_hit {
                    self.bloods.push(BloodSplatter::new(self.world.player.obj.clone()));
                    s.mplayer.play(ctx, Sound::Hit)?;

                    if self.world.player.health.is_dead() {
                        s.switch(StateSwitch::Lose(Box::new(Statistics{
                            hits: self.bloods.len(),
                            misses: self.misses,
                            enemies_left: self.world.enemies.len(),
                            health_left: self.initial.0,
                            level: self.level.clone(),
                            weapon: self.initial.1,
                        })));
                        s.mplayer.play(ctx, Sound::Death)?;
                    } else {
                        s.mplayer.play(ctx, Sound::Hurt)?;
                    }
                }
                for i in enemy_hits {
                    let enemy = &self.world.enemies[i];
                    s.mplayer.play(ctx, Sound::Hit)?;

                    self.bloods.push(BloodSplatter::new(enemy.pl.obj.clone()));
                    if enemy.pl.health.is_dead() {
                        s.mplayer.play(ctx, Sound::Death)?;

                        let Enemy{pl: Player{wep, obj: Object{pos, ..}, ..}, ..}
                            = self.world.enemies.remove(i);
                        if let Some(wep) = wep {
                            self.world.weapons.push(wep.into_drop(pos));
                        }
                    } else {
                        if !enemy.behaviour.chasing() {
                            self.world.enemies[i].behaviour = Chaser::LookAround{
                                dir: grenade.obj.pos - enemy.pl.obj.pos
                            };
                        }
                        s.mplayer.play(ctx, Sound::Hurt)?;
                    }
                }
            }
        }
        for i in deads {
            self.world.grenades.remove(i);
        }

        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            let hit = bullet.update(&self.world.grid, &mut self.world.player, &mut *self.world.enemies);
            
            use crate::obj::bullet::Hit;

            match hit {
                Hit::None => (),
                Hit::Wall => {
                    s.mplayer.play(ctx, bullet.weapon.impact_snd)?;
                    let dir = angle_to_vec(bullet.obj.rot);
                    bullet.obj.pos += Vector2::new(5.*dir.x.signum(), 5.*dir.y.signum());
                    self.holes.add(bullet.obj.drawparams());
                    self.misses += 1;
                    deads.push(i);
                }
                Hit::Player => {
                    deads.push(i);
                    self.bloods.push(BloodSplatter::new(bullet.obj.clone()));
                    s.mplayer.play(ctx, Sound::Hit)?;

                    if self.world.player.health.is_dead() {
                        s.switch(StateSwitch::Lose(Box::new(Statistics{
                            hits: self.bloods.len(),
                            misses: self.misses,
                            enemies_left: self.world.enemies.len(),
                            health_left: self.initial.0,
                            level: self.level.clone(),
                            weapon: self.initial.1,
                        })));
                        s.mplayer.play(ctx, Sound::Death)?;
                    } else {
                        s.mplayer.play(ctx, Sound::Hurt)?;
                    }
                }
                Hit::Enemy(e) => {
                    deads.push(i);
                    let enemy = &self.world.enemies[e];
                    s.mplayer.play(ctx, Sound::Hit)?;

                    self.bloods.push(BloodSplatter::new(bullet.obj.clone()));
                    if enemy.pl.health.is_dead() {
                        s.mplayer.play(ctx, Sound::Death)?;

                        let Enemy{pl: Player{wep, obj: Object{pos, ..}, ..}, ..}
                            = self.world.enemies.remove(e);
                        if let Some(wep) = wep {
                            self.world.weapons.push(wep.into_drop(pos));
                        }
                    } else {
                        if !enemy.behaviour.chasing() {
                            self.world.enemies[e].behaviour = Chaser::LookAround{
                                dir: bullet.obj.pos - enemy.pl.obj.pos
                            };
                        }
                        s.mplayer.play(ctx, Sound::Hurt)?;
                    }
                }
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
        }

        let mut deads = Vec::new();
        for (i, &intel) in self.world.intels.iter().enumerate().rev() {
            if (intel-self.world.player.obj.pos).norm() <= 15. {
                deads.push(i);
                s.mplayer.play(ctx, Sound::Hit)?;
            }
        }
        for i in deads {
            self.world.intels.remove(i);
        }
        let mut deads = Vec::new();
        for (i, pickup) in self.world.pickups.iter().enumerate().rev() {
            if (pickup.pos-self.world.player.obj.pos).norm() <= 15. {
                pickup.apply(&mut self.world.player.health);
                deads.push(i);
                s.mplayer.play(ctx, Sound::Hit)?;
            }
        }
        for i in deads {
            self.world.pickups.remove(i);
        }
        self.cur_pickup = None;
        for (i, weapon) in self.world.weapons.iter().enumerate().rev() {
            if (weapon.pos-self.world.player.obj.pos).norm() <= 29. {
                self.cur_pickup = Some(i);
                break
            }
        }

        // Define player velocity here already because enemies need it
        let player_vel = Vector2::new(hor(&ctx), ver(&ctx));

        for enemy in self.world.enemies.iter_mut() {
            if enemy.can_see(self.world.player.obj.pos, &self.world.grid) {
                enemy.behaviour = Chaser::LastKnown{
                    pos: self.world.player.obj.pos,
                    vel: player_vel,
                };

                if let Some(wep) = &mut enemy.pl.wep {
                    if let Some(bm) = wep.shoot(ctx, &mut s.mplayer)? {
                        let pos = enemy.pl.obj.pos + 20. * angle_to_vec(enemy.pl.obj.rot);
                        let mut bul = Object::new(pos);
                        bul.rot = enemy.pl.obj.rot;

                        self.world.bullets.push(bm.make(bul));
                    }
                }
            }
            enemy.update(ctx, &mut s.mplayer)?;
        }

        let speed = if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
            200.
        } else {
            100.
        };
        if let Some(wep) = &mut self.world.player.wep {
            wep.update(ctx, &mut s.mplayer)?;
            if wep.cur_clip > 0 && mouse::button_pressed(ctx, MouseButton::Left) && wep.weapon.fire_mode.is_auto() {
                if let Some(bm) = wep.shoot(ctx, &mut s.mplayer)? {
                    let pos = self.world.player.obj.pos + 20. * angle_to_vec(self.world.player.obj.rot);
                    let mut bul = Object::new(pos);
                    bul.rot = self.world.player.obj.rot;

                    self.world.bullets.push(bm.make(bul));
                }
            }
        }
        self.world.player.obj.move_on_grid(player_vel, speed, &self.world.grid);

        let game_won = match self.world.exit {
            Some(p) => self.world.intels.is_empty() && (p - self.world.player.obj.pos).norm() < 32.,
            None => self.world.enemies.is_empty(),
        };

        if game_won && self.victory_time <= 0. {
            s.mplayer.play(ctx, Sound::Victory)?;
            self.victory_time += DELTA;
        } else if self.victory_time > 0. {
            self.victory_time += DELTA;
        }
        if self.victory_time >= 2. {
            s.switch(StateSwitch::Win(Box::new(Statistics{
                level: self.level.clone(),
                hits: self.bloods.len(),
                misses: self.misses,
                enemies_left: self.world.enemies.len(),
                health_left: self.world.player.health,
                weapon: self.world.player.wep,
            })));
        }
        Ok(())
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let dist = s.mouse - s.offset - self.world.player.obj.pos;

        self.hud.update_bars(ctx, &self.world.player)?;

        self.world.player.obj.rot = angle_from_vec(dist);

        // Center the camera on the player
        let p = self.world.player.obj.pos;
        s.focus_on(p);
        Ok(())
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.world.grid.draw(ctx, &s.assets)?;

        self.holes.draw(ctx, Default::default())?;

        for &intel in &self.world.intels {
            let drawparams = graphics::DrawParam {
                dest: intel.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, "common/intel");
            graphics::draw(ctx, &*img, drawparams)?;
        }
        for decoration in &self.world.decorations {
            decoration.draw(ctx, &s.assets, WHITE)?;
        }

        for blood in &self.bloods {
            blood.draw(ctx, &s.assets)?;
        }

        for pickup in &self.world.pickups {
            let drawparams = graphics::DrawParam {
                dest: pickup.pos.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, pickup.pickup_type.spr);
            graphics::draw(ctx, &*img, drawparams)?;
        }
        for wep in &self.world.weapons {
            let drawparams = graphics::DrawParam {
                dest: wep.pos.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, wep.weapon.entity_sprite);
            graphics::draw(ctx, &*img, drawparams)?;
        }

        self.world.player.draw_player(ctx, &s.assets)?;

        for enemy in &self.world.enemies {
            enemy.draw(ctx, &s.assets, WHITE)?;
        }
        for bullet in &self.world.bullets {
            bullet.draw(ctx, &s.assets)?;
        }
        for grenade in &self.world.grenades {
            grenade.draw(ctx, &s.assets)?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.hud.draw(ctx)?;

        self.hp_text.draw_text(ctx)?;
        self.arm_text.draw_text(ctx)?;
        self.reload_text.draw_text(ctx)?;
        self.wep_text.draw_text(ctx)?;
        self.status_text.draw_center(ctx)?;

        let drawparams = graphics::DrawParam {
            dest: s.mouse.into(),
            offset: Point2::new(0.5, 0.5).into(),
            color: RED,
            .. Default::default()
        };
        let img = s.assets.get_img(ctx, "common/crosshair");
        graphics::draw(ctx, &*img, drawparams)
    }
    fn mouse_up(&mut self, s: &mut State, ctx: &mut Context, btn: MouseButton) {
        match btn {
            MouseButton::Left => {
                if let Some(wep) = &mut self.world.player.wep {
                    if let Some(bm) = wep.shoot(ctx, &mut s.mplayer).unwrap() {
                        let pos = self.world.player.obj.pos + 20. * angle_to_vec(self.world.player.obj.rot);
                        let mut bul = Object::new(pos);
                        bul.rot = self.world.player.obj.rot;

                        self.world.bullets.push(bm.make(bul));
                    }
                }
            }
            MouseButton::Right => {
                if let Some(gm) = self.world.player.utilities.throw_grenade(ctx, &mut s.mplayer).unwrap() {
                    let pos = self.world.player.obj.pos + 20. * angle_to_vec(self.world.player.obj.rot);
                    let mut gren = Object::new(pos);
                    gren.rot = self.world.player.obj.rot;

                    self.world.grenades.push(gm.make(gren));
                }
            }
            _ => (),
        }
    }
    fn key_up(&mut self, s: &mut State, ctx: &mut Context, keycode: KeyCode) {
        use self::KeyCode::*;
        match keycode {
            R => {
                if let Some(wep) = &mut self.world.player.wep {
                    wep.reload(ctx, &mut s.mplayer).unwrap()
                } else {
                    self.world.bullets.push(crate::obj::bullet::Bullet{obj: self.world.player.obj.clone(), weapon: &crate::obj::weapon::WEAPONS[0]});
                }
            },
            F => {
                if let Some(i) = self.cur_pickup {
                    self.world.player.wep = Some(WeaponInstance::from_drop(
                        if let Some(wep) = self.world.player.wep {
                            let w = wep.into_drop(self.world.player.obj.pos);
                            std::mem::replace(&mut self.world.weapons[i], w)
                        } else {
                            self.world.weapons.remove(i)
                        }
                    ));
                    self.cur_pickup = None;
                }
            },
            _ => (),
        }
    }

    fn get_world(&self) -> Option<&World> {
        Some(&self.world)
    }
    fn get_mut_world(&mut self) -> Option<&mut World> {
        Some(&mut self.world)
    }
}

#[derive(Debug)]
pub struct Hud {
    hud_bar: Mesh,
    hp_bar: Mesh,
    armour_bar: Mesh,
    loading_bar: Mesh,
}

impl Hud {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let hud_bar = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 1., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 29., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 57., w: 102., h: 26.}, graphics::BLACK)
            .build(ctx)?;

        let hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: 0., h: 24.}, GREEN)?;
        let armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: 0., h: 24.}, BLUE)?;
        let loading_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: 0., h: 24.}, RED)?;

        Ok(Hud{
            hud_bar,
            hp_bar,
            armour_bar,
            loading_bar,
        })
    }
    pub fn update_bars(&mut self, ctx: &mut Context, p: &Player) -> GameResult<()> {
        self.hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: p.health.hp.limit(0., 100.), h: 24.}, GREEN)?;
        self.armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: p.health.armour.limit(0., 100.), h: 24.}, BLUE)?;
        self.loading_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: p.wep.map(|m| m.loading_time).unwrap_or(0.).limit(0., 1.)*100., h: 24.}, RED)?;

        Ok(())
    }
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.hud_bar.draw(ctx, Default::default())?;
        self.hp_bar.draw(ctx, Default::default())?;
        self.armour_bar.draw(ctx, Default::default())?;
        self.loading_bar.draw(ctx, Default::default())
    }
}