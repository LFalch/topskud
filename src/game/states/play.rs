use std::f32::consts::FRAC_1_SQRT_2 as COS_45_D;
use topskud::{
    DELTA,
    ext::FloatExt,
    util::{
        BLUE, GREEN, RED,
        angle_to_vec, angle_from_vec,
        ver, hor, iterate_and_kill_afterwards, iterate_and_kill_afterwards_mut, iterate_and_kill_one_mut,
    },
    io::tex::PosText,
    obj::{
        Object,
        bullet::Bullet,
        decal::Decal,
        pickup::Pickup,
        player::{Player, WepSlots, ActiveSlot},
        enemy::Enemy,
        health::Health,
        weapon::{self, WeaponInstance},
        grenade::GrenadeUpdate,
    },
    world::{Level, Statistics, World},
};
use crate::game::{
    State, GameState, StateSwitch,
    event::{Event::{self, Key, Mouse}, MouseButton, KeyCode}
};
use ggez::{
    Context, GameResult,
    graphics::{
        self, Drawable, DrawMode, Rect,
        Color, DrawParam,
        MeshBuilder, Mesh,
        Canvas,
    },
    input::{
        keyboard::KeyMods,
        mouse,
    },
};

use rand::{thread_rng, prelude::SliceRandom, Rng};

pub fn new_blood(mut obj: Object) -> Decal {
    obj.pos += 16. * angle_to_vec(obj.rot);
    Decal {
        obj,
        spr: [
            "common/blood1",
            "common/blood2",
            "common/blood2",
            "common/blood3",
            "common/blood3",
        ].choose(&mut thread_rng()).copied().map(Into::into).unwrap(),
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
    cur_pickup: Option<usize>,
    victory_time: f32,
    time: usize,
    initial: (Health, WepSlots),
    level: Level,
}

impl Play {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State, level: Level, pl: Option<(Health, WepSlots)>) -> GameResult<Box<dyn GameState>> {
        mouse::set_cursor_hidden(ctx, true);

        let mut player = Player::from_point(level.start_point.unwrap_or_else(|| point!(500., 500.)));
        if let Some((h, w)) = pl {
            player = player.with_health(h).with_weapon(w);
        };

        Ok(Box::new(
            Play {
                level: level.clone(),
                initial: (player.health, player.wep.clone()),
                hp_text: s.assets.text(point!(4., 4.)).and_text("100"),
                arm_text: s.assets.text(point!(4., 33.)).and_text("100"),
                reload_text: s.assets.text(point!(4., 62.)).and_text("0.0").and_text("s"),
                wep_text: WeaponInstance::weapon_text(point!(2., 87.), &s.assets),
                status_text: s.assets.text(point!(s.width as f32 / 2., s.height as f32 / 2. + 32.)).and_text("").centered(),
                hud: Hud::new(ctx)?,
                time: 0,
                victory_time: 0.,
                cur_pickup: None,
                world: {
                    let mut world = World {
                        enemies: level.enemies,
                        bullets: Vec::new(),
                        grenades: Vec::new(),
                        decal_queue: level.decals.clone(),
                        weapons: level.weapons,
                        player,
                        canvas: None,
                        palette: level.palette,
                        grid: level.grid,
                        exit: level.exit,
                        intels: level.intels,
                        pickups: level.pickups.into_iter().map(|(p, i)| Pickup::new(p, i)).collect(),
                    };
                    world.enemy_pickup();
                    world.player_pickup();

                    if world.player.wep.get_active().is_none() {
                        warn!("player has no weapon");
                    }

                    for enemy_pos in world.enemies.iter().filter_map(|enemy| if enemy.pl.wep.get_active().is_none() {Some(enemy.pl.obj.pos)}else{None}) {
                        warn!("enemy at {:.2} has no weapon", enemy_pos)
                    }

                    world
                },
            }
        ))
    }
    pub fn add_decal(&mut self, decal: Decal) {
        self.world.decal_queue.push(decal);
    }
}

impl GameState for Play {
    #[allow(clippy::cognitive_complexity)]
    fn update(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        self.hp_text.update(0, format!("{:02.0}", self.world.player.health.hp))?;
        self.arm_text.update(0, format!("{:02.0}", self.world.player.health.armour))?;
        if let Some(wep) = self.world.player.wep.get_active() {
            self.reload_text.update(0, format!("{:.1}", wep.loading_time))?;
            wep.update_text(&mut self.wep_text)?;
        }
        if let Some(i) = self.cur_pickup {
            // TODO change text to say what's being swapped out
            self.status_text.text.fragments_mut()[0]= format!("Press F to pick up {}", self.world.weapons[i]).into();
        } else {
            self.status_text.update(0, "")?;
        }

        iterate_and_kill_afterwards_mut(&mut self.world.grenades, |grenade| {
            let g_update = grenade.update(ctx, &self.world.palette, &self.world.grid, &mut self.world.player, &mut *self.world.enemies)?;

            Ok(match g_update {
                GrenadeUpdate::Explosion{player_hit, enemy_hits} => {
                    s.mplayer.play(ctx, "boom")?;

                    self.world.decal_queue.push(Decal {
                        obj: grenade.obj.clone(),
                        spr: "common/blast",
                    });
                    if player_hit {
                        self.world.decal_queue.push(new_blood(self.world.player.obj.clone()));
                        s.mplayer.play(ctx, "hit")?;

                        if self.world.player.health.is_dead() {
                            s.switch(StateSwitch::Lose(Box::new(Statistics{
                                time: self.time,
                                enemies_left: self.world.enemies.len(),
                                health_left: self.initial.0,
                                level: self.level.clone(),
                                weapon: self.initial.1.clone(),
                            })));
                            s.mplayer.play(ctx, "death")?;
                        } else {
                            s.mplayer.play(ctx, "hurt")?;
                        }
                    }
                    for i in enemy_hits {
                        let enemy = &self.world.enemies[i];
                        s.mplayer.play(ctx, "hit")?;

                        self.world.decal_queue.push(new_blood(enemy.pl.obj.clone()));
                        if enemy.pl.health.is_dead() {
                            s.mplayer.play(ctx, "death")?;

                            let Enemy{pl: Player{wep, obj: Object{pos, ..}, ..}, ..}
                                = self.world.enemies.remove(i);
                            for wep in wep {
                                self.world.weapons.push(wep.into_drop(pos));
                            }
                        } else {
                            if !enemy.behaviour.chasing() {
                                let cur_pos = enemy.pl.obj.pos;
                                self.world.enemies[i].behaviour.go_to_then_go_back(cur_pos, grenade.obj.pos);
                            }
                            s.mplayer.play(ctx, "hurt")?;
                        }
                    }
                    false
                }
                GrenadeUpdate::Dead => true,
                GrenadeUpdate::None => false
            })
        })?;

        iterate_and_kill_afterwards_mut(&mut self.world.bullets, |bullet| {
            let hit = bullet.update(&self.world.palette, &self.world.grid, &mut self.world.player, &mut *self.world.enemies);
            let mut dead = false;

            use topskud::obj::bullet::Hit;

            match hit {
                Hit::None => (),
                Hit::Wall => {
                    s.mplayer.play(ctx, &bullet.weapon.impact_snd)?;
                    let dir = angle_to_vec(bullet.obj.rot);
                    bullet.obj.pos += vector!(5.*dir.x.signum(), 5.*dir.y.signum());
                    self.world.decal_queue.push(Decal {
                        obj: bullet.obj.clone(),
                        spr: "common/hole",
                    });
                    dead = true;
                }
                Hit::Player => {
                    dead = true;
                    self.world.decal_queue.push(new_blood(bullet.obj.clone()));
                    s.mplayer.play(ctx, "hit")?;

                    if self.world.player.health.is_dead() {
                        s.switch(StateSwitch::Lose(Box::new(Statistics{
                            time: self.time,
                            enemies_left: self.world.enemies.len(),
                            health_left: self.initial.0,
                            level: self.level.clone(),
                            weapon: self.initial.1.clone(),
                        })));
                        s.mplayer.play(ctx, "death")?;
                    } else {
                        s.mplayer.play(ctx, "hurt")?;
                    }
                }
                Hit::Enemy(e) => {
                    dead = true;
                    let enemy = &self.world.enemies[e];
                    s.mplayer.play(ctx, "hit")?;

                    self.world.decal_queue.push(new_blood(bullet.obj.clone()));
                    if enemy.pl.health.is_dead() {
                        s.mplayer.play(ctx, "death")?;

                        let Enemy{pl: Player{wep, obj: Object{pos, ..}, ..}, ..}
                            = self.world.enemies.remove(e);
                        for wep in wep {
                            self.world.weapons.push(wep.into_drop(pos));
                        }
                    } else {
                        if !enemy.behaviour.chasing() {
                            let cur_pos = enemy.pl.obj.pos;
                            self.world.enemies[e].behaviour.go_to_then_go_back(cur_pos, bullet.obj.pos);
                        }
                        s.mplayer.play(ctx, "hurt")?;
                    }
                }
            }
            Ok(dead)
        })?;

        iterate_and_kill_afterwards(&mut self.world.intels, |&intel| {
            Ok(if (intel-self.world.player.obj.pos).norm() <= 15. {
                s.mplayer.play(ctx, "hit")?;
                true
            } else { false })
        })?;
        iterate_and_kill_afterwards(&mut self.world.pickups, |pickup| {
            Ok(if (pickup.pos-self.world.player.obj.pos).norm() <= 15. && pickup.apply(&mut self.world.player.health) {
                s.mplayer.play(ctx, "hit")?;
                true
            } else { false })
        })?;
        self.cur_pickup = None;
        for (i, weapon) in self.world.weapons.iter().enumerate().rev() {
            if (weapon.pos-self.world.player.obj.pos).norm() <= 29. {
                self.cur_pickup = Some(i);
                break
            }
        }

        // Define player velocity here already because enemies need it
        let player_vel = vector!(hor(&ctx), ver(&ctx));

        let &mut World {ref grid, ref palette, ref mut enemies, ref player, ref mut bullets, ..} = &mut self.world;

        for enemy in enemies.iter_mut() {
            if enemy.can_see(player.obj.pos, palette, grid) {
                // If an enemy can see the player, they will chase them and shoot

                // enemy.behaviour.chase_then_wander(self.world.player.obj.pos);
                enemy.behaviour.path_then_wander(vec![player.obj.pos, player.obj.pos+16.*player_vel]);

                if let Some(wep) = enemy.pl.wep.get_active_mut() {
                    if let Some(bm) = wep.shoot(ctx, &mut s.mplayer)? {
                        let pos = enemy.pl.obj.pos + 20. * angle_to_vec(enemy.pl.obj.rot);
                        let mut bul = Object::new(pos);
                        bul.rot = enemy.pl.obj.rot;

                        for bullet in bm.make(bul) {
                            bullets.push(bullet);
                        }
                    }
                }
            }
            let from = enemy.pl.obj.pos;

            enemy.update(ctx, &mut s.mplayer, || {
                let dir = thread_rng().gen_range(0. .. 2. * std::f32::consts::PI);
                let length = thread_rng().gen_range(0. ..= 1.);

                const MAX_DIST: f32 = 128.;

                let cast = grid.ray_cast(palette, from, MAX_DIST * angle_to_vec(dir), true);
                let p = cast.into_point();

                from + length * (p - from)
            })?;
        }

        let speed = if !ctx.keyboard.is_mod_active(KeyMods::SHIFT) {
            200.
        } else {
            100.
        };
        if let Some(wep) = self.world.player.wep.get_active_mut() {
            wep.update(ctx, &mut s.mplayer)?;
            if wep.cur_clip > 0 && ctx.mouse.button_pressed(MouseButton::Left) && wep.weapon.fire_mode.is_auto() {
                if let Some(bm) = wep.shoot(ctx, &mut s.mplayer)? {
                    let pos = self.world.player.obj.pos + 20. * angle_to_vec(self.world.player.obj.rot);
                    let mut bul = Object::new(pos);
                    bul.rot = self.world.player.obj.rot;

                    for bullet in bm.make(bul) {
                        self.world.bullets.push(bullet);
                    }
                }
            }
        }
        self.world.player.obj.move_on_grid(player_vel, speed, &self.world.palette, &self.world.grid);

        let game_won = match self.world.exit {
            Some(p) => self.world.intels.is_empty() && (p - self.world.player.obj.pos).norm() < 32.,
            None => self.world.enemies.is_empty(),
        };

        if game_won && self.victory_time <= 0. {
            s.mplayer.play(ctx, "victory")?;
            self.victory_time += DELTA;
        } else if self.victory_time > 0. {
            self.victory_time += DELTA;
        } else {
            self.time += 1;
        }
        if self.victory_time >= 2. {
            s.switch(StateSwitch::Win(Box::new(Statistics{
                level: self.level.clone(),
                time: self.time,
                enemies_left: self.world.enemies.len(),
                health_left: self.world.player.health,
                weapon: self.world.player.wep.clone(),
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

    fn draw(&mut self, s: &State, canvas: &mut Canvas, ctx: &mut Context) -> GameResult<()> {
        self.world.draw_world(ctx, canvas, &s.assets)?;

        for &intel in &self.world.intels {
            let drawparams = graphics::DrawParam::default()
                .dest(intel)
                .offset(point!(0.5, 0.5));
            let img = s.assets.get_img("common/intel");
            canvas.draw(&*img, drawparams);
        }

        for pickup in &self.world.pickups {
            let drawparams = graphics::DrawParam::default()
                .dest(pickup.pos)
                .offset(point!(0.5, 0.5));
            let img = s.assets.get_img(pickup.pickup_type.spr);
            canvas.draw(&*img, drawparams);
        }
        for wep in &self.world.weapons {
            let drawparams = graphics::DrawParam::default()
                .dest(wep.pos)
                .offset(point!(0.5, 0.5));
            let img = s.assets.get_img(&wep.weapon.entity_sprite);
            canvas.draw(&*img, drawparams);
        }

        self.world.player.draw_player(canvas, &s.assets);

        for enemy in &self.world.enemies {
            enemy.draw(canvas, &s.assets, Color::WHITE);
        }
        for bullet in &self.world.bullets {
            bullet.draw(canvas, &s.assets);
        }
        for grenade in &self.world.grenades {
            grenade.draw(canvas, &s.assets);
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, canvas: &mut Canvas, ctx: &mut Context) -> GameResult<()> {
        self.hud.draw(canvas)?;

        self.hp_text.draw_text(canvas);
        self.arm_text.draw_text(canvas);
        self.reload_text.draw_text(canvas);
        self.wep_text.draw_text(canvas);
        self.status_text.draw_text(canvas);

        {
            let drawparams = DrawParam::from(point![104., 2.]);
            let img = s.assets.get_img("weapons/knife");
            canvas.draw(&*img, drawparams);
        }
        if let Some(holster_wep) = &self.world.player.wep.holster {
            let drawparams = DrawParam::from(point![137., 2.]);
            let img = s.assets.get_img(&holster_wep.weapon.entity_sprite);
            canvas.draw(&*img, drawparams);
        }
        if let Some(holster_wep) = &self.world.player.wep.holster2 {
            let drawparams = DrawParam::from(point![104., 35.]);
            let img = s.assets.get_img(&holster_wep.weapon.entity_sprite);
            canvas.draw(&*img, drawparams);
        }
        if let Some(sling_wep) = &self.world.player.wep.sling {
            let drawparams = DrawParam::from(point![153., 35.]).offset(point!(0.5, 0.));
            let img = s.assets.get_img(&sling_wep.weapon.entity_sprite);
            canvas.draw(&*img, drawparams);
        }
        let selection = Mesh::new_rectangle(ctx, DrawMode::stroke(2.), RECTS[self.world.player.wep.active as u8 as usize], Color{r: 1., g: 1., b: 0., a: 1.})?;
        canvas.draw(&selection, DrawParam::default());

        let drawparams = graphics::DrawParam::default()
            .dest(s.mouse)
            .offset(point!(0.5, 0.5))
            .color(RED);
        let img = s.assets.get_img("common/crosshair");
        canvas.draw(&*img, drawparams);
        Ok(())
    }
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        use self::KeyCode::*;
        match event {
            Key(Q) => self.world.player.wep.switch(self.world.player.wep.last_active),
            Key(Key1) | Key(Numpad1) => self.world.player.wep.switch(ActiveSlot::Knife),
            Key(Key2) | Key(Numpad2) => self.world.player.wep.switch(ActiveSlot::Holster),
            Key(Key3) | Key(Numpad3) => self.world.player.wep.switch(ActiveSlot::Holster2),
            Key(Key4) | Key(Numpad4) => self.world.player.wep.switch(ActiveSlot::Sling),
            Key(G) => {
                if let Some(wep) = self.world.player.wep.take_active() {
                    self.world.weapons.push(wep.into_drop(self.world.player.obj.pos));
                }
            }
            Key(R) => {
                if let Some(wep) = self.world.player.wep.get_active_mut() {
                    wep.reload(ctx, &mut s.mplayer).unwrap()
                } else {
                    let weapon = &weapon::WEAPONS["glock"];
                    self.world.bullets.push(Bullet{obj: self.world.player.obj.clone(), vel: vector!(weapon.bullet_speed, 0.), weapon});
                }
            },
            Key(F) => {
                if let Some(i) = self.cur_pickup {
                    if let Some(new_drop) = self.world.player.wep.add_weapon(WeaponInstance::from_drop(self.world.weapons.remove(i))) {
                        self.world.weapons.push(new_drop.into_drop(self.world.player.obj.pos));
                    }
                    self.cur_pickup = None;
                }
            },
            Mouse(MouseButton::Left) | Key(Space) => {
                if let Some(wep) = self.world.player.wep.get_active_mut() {
                    if let Some(bm) = wep.shoot(ctx, &mut s.mplayer).unwrap() {
                        let pos = self.world.player.obj.pos + 20. * angle_to_vec(self.world.player.obj.rot);
                        let mut bul = Object::new(pos);
                        bul.rot = self.world.player.obj.rot;

                        for bullet in bm.make(bul) {
                            self.world.bullets.push(bullet);
                        }
                    }
                } else {
                    // TODO do knives with bullets too
                    let player = &mut self.world.player;
                    let mut backstab = false;

                    let killed_enemy = iterate_and_kill_one_mut(&mut self.world.enemies, |enemy| {
                        let dist = player.obj.pos-enemy.pl.obj.pos;
                        let dist_len = dist.norm();
                        if dist_len < 44. {
                            backstab = angle_to_vec(enemy.pl.obj.rot).dot(&dist) / dist_len < COS_45_D;

                            self.world.decal_queue.push(new_blood(enemy.pl.obj.clone()));
                            enemy.pl.health.weapon_damage(if backstab { 165. } else { 33. }, 0.92);

                            // Kill enemy if dead
                            enemy.pl.health.is_dead()
                        } else { false }
                    });
                    if let Some(Enemy{pl: Player{wep, obj: Object{pos, ..}, ..}, ..}) = killed_enemy {
                        s.mplayer.play(ctx, "death").unwrap();

                        for wep in wep {
                            self.world.weapons.push(wep.into_drop(pos));
                        }
                    }

                    s.mplayer.play(ctx, if backstab {"shuk"} else {"hling"}).unwrap();
                }
            }
            Mouse(MouseButton::Right) => {
                if let Some(gm) = self.world.player.wep.utilities.throw_grenade(ctx, &mut s.mplayer).unwrap() {
                    let pos = self.world.player.obj.pos + 20. * angle_to_vec(self.world.player.obj.rot);
                    let mut gren = Object::new(pos);
                    gren.rot = self.world.player.obj.rot;

                    self.world.grenades.push(gm.make(gren));
                }
            }
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

const RECTS: [Rect; 4] = [
    Rect{x:104.,y:2.,h: 32., w: 32.},
    Rect{x:137.,y:2.,h: 32., w: 32.},
    Rect{x:104.,y:35.,h: 32., w: 32.},
    Rect{x:137.,y:35.,h: 32., w: 32.}
];

impl Hud {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut hud_bar_builder = MeshBuilder::new();
        hud_bar_builder
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 1., w: 102., h: 26.}, Color::BLACK)?
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 29., w: 102., h: 26.}, Color::BLACK)?
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 57., w: 102., h: 26.}, Color::BLACK)?
            .rectangle(DrawMode::fill(), Rect{x:104.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?
            .rectangle(DrawMode::fill(), Rect{x:137.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?
            .rectangle(DrawMode::fill(), Rect{x:104.,y:35.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?
            .rectangle(DrawMode::fill(), Rect{x:137.,y:35.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?
            ;

        let hud_bar = Mesh::from_data(ctx, hud_bar_builder.build());

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
        self.loading_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: p.wep.get_active().map(|m| m.loading_time).unwrap_or(0.).limit(0., 1.)*100., h: 24.}, RED)?;

        Ok(())
    }
    pub fn draw(&self, canvas: &mut Canvas) -> GameResult<()> {
        self.hud_bar.draw(canvas, DrawParam::default());
        self.hp_bar.draw(canvas, DrawParam::default());
        self.armour_bar.draw(canvas, DrawParam::default());
        self.loading_bar.draw(canvas, DrawParam::default());
        Ok(())
    }
}