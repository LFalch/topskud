use crate::{
    util::{
        ver,
        hor,
        sstr,
        TRANS,
        Point2
    },
    io::tex::PosText,
    ext::BoolExt,
    game::{
        DELTA, Content, GameState, State, StateSwitch,
        world::{Grid, Level, Palette},
        event::{Event::{self, Key, Mouse}, MouseButton as Mb, KeyCode}
    },
    obj::{Object, enemy::Enemy, decal::Decal, pickup::PICKUPS, weapon::WEAPONS}
};
use ggez::{
    Context, GameResult,
    graphics::{self, Color, Rect, DrawMode, DrawParam, Mesh, Canvas},
    error::GameError,
    input::{
        keyboard::{KeyMods},
    },
};

use std::{path::PathBuf, iter};
use std::io::Read;
use std::fs::File;

#[derive(Debug, PartialEq, Clone)]
enum Tool {
    Inserter(Insertion),
    Selector(Selection),
}

#[derive(Debug, Clone, Copy)]
enum Insertion {
    Material(u8),
    Intel,
    Enemy{rot: f32},
    Waypoint(usize),
    Pickup(u8),
    Weapon(&'static str),
    Decal{spr: &'static str, rot: f32},
    Exit,
}
impl Insertion {
    fn get_spr(&self) -> &str {
        use Insertion::*;
        match *self {
            Material(_) => panic!("Get it yourself. I don't have the palette"),
            Intel => "common/intel",
            Enemy{..} => "common/enemy",
            Waypoint(..) => "common/cursor",
            Exit => "common/goal",
            Pickup(i) => PICKUPS[i as usize].spr,
            Weapon(wep) => &*WEAPONS[wep].entity_sprite, 
            Decal{spr, ..} => spr,
        }
    }
}
impl ::std::cmp::PartialEq for Insertion {
    fn eq(&self, rhs: &Self) -> bool {
        use self::Insertion::*;
        match (self, rhs) {
            (Material(m), Material(n)) if m == n => true,
            (Intel, Intel) => true,
            (Enemy{..}, Enemy{..}) => true,
            (Pickup(i), Pickup(j)) if i == j => true,
            (Weapon(i), Weapon(j)) if i == j => true,
            (Decal{spr, ..}, Decal{spr: spr2, ..}) if spr == spr2 => true,
            (Exit, Exit) => true,
            _ => false
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
struct Selection {
    exit: bool,
    enemies: Vec<usize>,
    waypoints: Vec<(usize, usize)>,
    intels: Vec<usize>,
    pickups: Vec<usize>,
    weapons: Vec<usize>,
    decals: Vec<usize>,
    moving: Option<Point2>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EditorFile {
    palettes: EditorPalettes,
}

#[derive(Debug, Serialize, Deserialize)]
struct EditorPalettes {
    materials: Vec<String>,
    weapons: Vec<String>,
    decals: Vec<String>,
}

/// The state of the game
pub struct Editor {
    save: PathBuf,
    pos: Point2,
    level: Level,
    current: Tool,
    mat_text: PosText,
    entities_bar: InsertionBar,
    extra_bar: InsertionBar,
    draw_visibility_cones: bool,
    rotation_speed: f32,
    snap_on_grid: bool,
}


struct InsertionBar {
    ent_text: PosText,
    palette: Box<[Insertion]>,
}

impl InsertionBar {
    fn new(p: Point2, s: &State, text: &str, palette: Box<[Insertion]>) -> Self {
        let ent_text = s.assets.text(p).and_text(text);
        Self {
            ent_text,
            palette
        }
    }
    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas, s: &State, cur: Option<Insertion>) -> GameResult<()> {
        let mut dest = self.ent_text.pos + vector!(98., 16.);

        let mut drawparams = graphics::DrawParam::default()
            .dest(dest)
            .offset(point!(0.5, 0.5));

        for ins in &*self.palette {
            if let Some(cur) = cur {
                if ins == &cur {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), dest, 17., 0.5, YELLOW)?;
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
            let img = s.assets.get_img(ins.get_spr());
            canvas.draw(&*img, drawparams);
            dest.x += 34.; 
            drawparams = drawparams.dest(dest);
        }
        Ok(())
    }
    fn click(&self, mouse: Point2) -> Option<Insertion> {
        if mouse.y >= self.ent_text.pos.y && mouse.y < self.ent_text.pos.y+32. {
            let mut range = self.ent_text.pos.x + 82.;
            for ins in &*self.palette {
                if mouse.x >= range && mouse.x < range + 32. {
                    return Some(*ins);
                }
                range += 34.;
            }
        }
        None
    }
}

impl Editor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(s: &State, level: Option<Level>) -> GameResult<Box<dyn GameState>> {
        let mat_text = s.assets.text(point!(2., 18.0)).and_text("Materials:");
        let mut entities = vec![
            Insertion::Enemy{rot: 0.},
            Insertion::Exit,
            Insertion::Intel,
            Insertion::Pickup(0),
            Insertion::Pickup(1),
            Insertion::Pickup(2),
            Insertion::Pickup(3),
            Insertion::Pickup(4),
            Insertion::Pickup(5),
        ];

        let EditorFile{palettes: EditorPalettes{materials, weapons, decals}} = {
            let mut file = File::open("resources/editor.toml").unwrap();
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            
            toml::from_str(&s).unwrap()
        };
        entities.extend(weapons.into_iter().map(|wep| Insertion::Weapon(sstr(wep))));
        entities.extend(decals.into_iter().map(|dec| Insertion::Decal{rot: 0., spr: sstr(dec)}));

        let extra_entities = entities.drain(20..).collect();

        let entities_bar = InsertionBar::new(point!(392., 18.0), s, "Entitites:", entities.into_boxed_slice());
        let extra_bar = InsertionBar::new(point!(392., 52.0), s, "", extra_entities);

        let palette = Palette::new(materials.into_iter().map(sstr).collect());

        let save;
        if let Content::File(ref f) = s.content {
            save = f.clone();
        } else {
            return Err(GameError::ResourceLoadError("Cannot load editor without file".to_owned()));
        }

        let mut level = level
            .or_else(|| Level::load(&save).ok())
            .unwrap_or_else(|| Level::new(palette.clone(), 32, 32));
        level.palette = level.grid.migrate(&level.palette, palette);

        let x = f32::from(level.grid.width()) * 16.;
        let y = f32::from(level.grid.height()) * 16.;

        Ok(Box::new(Editor {
            save,
            pos: point!(x, y),
            current: Tool::Selector(Selection::default()),
            draw_visibility_cones: false,
            mat_text,
            entities_bar,
            extra_bar,
            level,
            rotation_speed: 0.,
            snap_on_grid: false,
        }))
    }
    fn mousepos(&self, s: &State) -> Point2 {
        let mut mp = s.mouse - s.offset;
        if self.snap_on_grid {
            mp.x = (mp.x / 32.).floor() * 32. + 16.;
            mp.y = (mp.y / 32.).floor() * 32. + 16.;
        }
        mp
    }
    fn displace<F: FnMut(&mut Point2)>(&mut self, f: F) {
        self.level.enemies
            .iter_mut()
            .flat_map(|e| iter::once(&mut e.pl.obj.pos).chain(&mut e.behaviour.path))
            .chain(self.level.exit.as_mut())
            .chain(self.level.start_point.as_mut())
            .chain(iter::once(&mut self.pos))
            .chain(self.level.intels.iter_mut())
            .chain(self.level.weapons.iter_mut().map(|w| &mut w.pos))
            .chain(self.level.pickups.iter_mut().map(|p| &mut p.0))
            .chain(self.level.decals.iter_mut().map(|d| &mut d.obj.pos))
            .for_each(f)
    }
}

const START_X: f32 = 103.;
const YELLOW: Color = Color{r: 1., g: 1., b: 0., a: 1.};

impl GameState for Editor {
    fn update(&mut self, _s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let speed = if ctx.keyboard.is_mod_active(KeyMods::SHIFT) { 315. } else { 175. };
        let v = speed * vector!(hor(ctx), ver(ctx));
        self.pos += v * DELTA;

        match self.current {
            Tool::Inserter(Insertion::Enemy{ref mut rot}) => *rot += self.rotation_speed * DELTA,
            Tool::Inserter(Insertion::Decal{ref mut rot, ..}) => *rot += self.rotation_speed * DELTA,
            _ => (),
        }
        Ok(())
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        if ctx.mouse.button_pressed(Mb::Left) && s.mouse.y > 64. {
            if let Tool::Inserter(Insertion::Material(mat)) = self.current {
                let (mx, my) = Grid::snap(s.mouse - s.offset);
                self.level.grid.insert(mx, my, mat);
            }
        }

        s.focus_on(self.pos);
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    fn draw(&mut self, s: &State, canvas: &mut Canvas, ctx: &mut Context) -> GameResult<()> {
        self.level.grid.draw(&self.level.palette, canvas, &s.assets);

        if let Tool::Inserter(Insertion::Material(mat)) = self.current {
            let (x, y) = Grid::snap(s.mouse-s.offset);
            let x = f32::from(x) * 32.;
            let y = f32::from(y) * 32.;
            self.level.palette.draw_mat(mat, canvas, &s.assets, x, y, graphics::DrawParam {
                color: TRANS,
                .. Default::default()
            });
        }

        if let Some(start) = self.level.start_point {
            let img = s.assets.get_img("common/start");
            canvas.draw(&*img, graphics::DrawParam::default()
                .dest(start)
                .offset(point!(0.5, 0.5)));
        }
        if let Some(exit) = self.level.exit {
            if let Tool::Selector(Selection{exit: true, ..}) = self.current {
                let mesh = Mesh::new_circle(ctx, DrawMode::fill(), exit, 17., 0.5, YELLOW).unwrap();
                canvas.draw(&mesh, DrawParam::default());
            }
            let drawparams = graphics::DrawParam::default()
                .dest(exit)
                .offset(point!(0.5, 0.5));
            let img = s.assets.get_img("common/goal");
            canvas.draw(&*img, drawparams);
        }

        for (i, &intel) in self.level.intels.iter().enumerate() {
            if let Tool::Selector(Selection{ref intels, ..}) = self.current {
                if intels.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), intel, 17., 0.5, YELLOW).unwrap();
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
            let drawparams = graphics::DrawParam::default()
                .dest(intel)
                .offset(point!(0.5, 0.5));

            let img = s.assets.get_img("common/intel");
            canvas.draw(&*img, drawparams);
        }

        for (i, enemy) in self.level.enemies.iter().enumerate() {
            if let Tool::Selector(Selection{ref enemies, ..})= self.current {
                if enemies.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), enemy.pl.obj.pos, 17., 0.5, YELLOW).unwrap();
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
            if self.draw_visibility_cones {
                enemy.draw_visibility_cone(ctx, canvas, 512.)?;
            }
            let mut points_lines = vec![enemy.pl.obj.pos];
            
            enemy.draw(canvas, &s.assets, Color::WHITE);
            for &waypoint in &enemy.behaviour.path {
                let img = s.assets.get_img("common/crosshair");
                canvas.draw(&*img, DrawParam::default().offset(point!(0.5, 0.5)).dest(waypoint).color(Color::YELLOW));
                points_lines.push(waypoint);
            }
            if points_lines.len() > 1 {
                if enemy.behaviour.cyclical_path {
                    points_lines.push(points_lines[1]);
                }
                let mesh = Mesh::new_line(ctx, &points_lines, 2., Color::BLUE)?;
                canvas.draw(&mesh, DrawParam::default());
            }
            if enemy.behaviour.cyclical_path {
                let img = s.assets.get_img("common/cyclic");
                canvas.draw(&*img, DrawParam::default().offset(point!(0.5, 0.5)).dest(enemy.pl.obj.pos));
            }
        }
        for (i, decal) in self.level.decals.iter().enumerate() {
            if let Tool::Selector(Selection{ref decals, ..})= self.current {
                if decals.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), decal.obj.pos, 17., 0.5, YELLOW)?;
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
            decal.draw(canvas, &s.assets, Color::WHITE);
        }

        // Draw init pick-up-ables on top of enemies so they're visible
        for (i, pickup) in self.level.pickups.iter().enumerate() {
            if let Tool::Selector(Selection{ref pickups, ..}) = self.current {
                if pickups.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), pickup.0, 17., 0.5, YELLOW)?;
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
            PICKUPS[pickup.1 as usize].draw(pickup.0, canvas, &s.assets);
        }
        for (i, weapon) in self.level.weapons.iter().enumerate() {
            if let Tool::Selector(Selection{ref weapons, ..}) = self.current {
                if weapons.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), weapon.pos, 17., 0.5, YELLOW)?;
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
            let drawparams = graphics::DrawParam::default()
                .dest(weapon.pos)
                .offset(point!(0.5, 0.5));

            let img = s.assets.get_img(&weapon.weapon.entity_sprite);
            canvas.draw(&*img, drawparams);
        }

        // Draw moving objects shadows
        if let Tool::Selector(ref selection @ Selection{moving: Some(_), ..}) = self.current {
            let mousepos = self.mousepos(s);
            let dist = mousepos - selection.moving.unwrap();

            for &i in &selection.enemies {
                let mut enem = self.level.enemies[i].clone();
                enem.pl.obj.pos += dist;
                enem.draw(canvas, &s.assets, TRANS);
            }
            for &i in &selection.intels {
                let drawparams = graphics::DrawParam::default()
                    .dest(self.level.intels[i] + dist)
                    .offset(point!(0.5, 0.5))
                    .color(TRANS);

                let img = s.assets.get_img("common/intel");
                canvas.draw(&*img, drawparams);
            }
            for &i in &selection.decals {
                let mut dec = self.level.decals[i].clone();
                dec.obj.pos += dist;
                dec.draw(canvas, &s.assets, TRANS);
            }
            for &i in &selection.pickups {
                let pickup = self.level.pickups[i];
                let drawparams = graphics::DrawParam::default()
                    .dest(pickup.0 + dist)
                    .offset(point!(0.5, 0.5))
                    .color(TRANS);
                let img = s.assets.get_img(PICKUPS[pickup.1 as usize].spr);
                canvas.draw(&*img, drawparams);
            }
            for &i in &selection.weapons {
                let drawparams = graphics::DrawParam::default()
                    .dest(self.level.weapons[i].pos + dist)
                    .offset(point!(0.5, 0.5))
                    .color(TRANS);
                let img = s.assets.get_img(&self.level.weapons[i].weapon.entity_sprite);
                canvas.draw(&*img, drawparams);
            }
            if selection.exit {
                if let Some(exit) = self.level.exit {
                    let drawparams = graphics::DrawParam::default()
                        .dest(exit + dist)
                        .offset(point!(0.5, 0.5))
                        .color(TRANS);
                    let img = s.assets.get_img("common/goal");
                    canvas.draw(&*img, drawparams);
                }
            }
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, canvas: &mut Canvas, ctx: &mut Context) -> GameResult<()> {
        let drawparams = graphics::DrawParam::default()
            .dest(self.mousepos(s) + s.offset)
            .rotation(0.)
            .offset(point!(0.5, 0.5))
            .color(TRANS);

        match self.current {
            Tool::Selector(_) => (),
            Tool::Inserter(Insertion::Waypoint(i)) => {
                let img = s.assets.get_img("common/crosshair");
                canvas.draw(&*img, drawparams.color(Color::BLUE));

                let last_waypoint = self.level.enemies[i].behaviour.path.last().copied().unwrap_or_else(|| self.level.enemies[i].pl.obj.pos);
                let next_pos = self.mousepos(s);

                if last_waypoint != next_pos {
                    let line = Mesh::new_line(ctx, &[self.mousepos(s) + s.offset, last_waypoint + s.offset], 2., Color::BLUE)?;
                    canvas.draw(&line, DrawParam::default());
                }
            }
            Tool::Inserter(Insertion::Material(_)) => (),
            Tool::Inserter(Insertion::Pickup(index)) => {
                let img = s.assets.get_img(PICKUPS[index as usize].spr);
                canvas.draw(&*img, drawparams);
            }
            Tool::Inserter(Insertion::Weapon(id)) => {
                let img = s.assets.get_img(&WEAPONS[id].entity_sprite);
                canvas.draw(&*img, drawparams);
            }
            Tool::Inserter(Insertion::Enemy{rot}) => {
                let img = s.assets.get_img("common/enemy");
                canvas.draw(&*img, drawparams.rotation(rot));
            }
            Tool::Inserter(Insertion::Decal{spr, rot}) => {
                let img = s.assets.get_img(spr);
                canvas.draw(&*img, drawparams.rotation(rot));
            }
            Tool::Inserter(Insertion::Exit) => {
                let img = s.assets.get_img("common/goal");
                canvas.draw(&*img, drawparams);
            }
            Tool::Inserter(Insertion::Intel) => {
                let img = s.assets.get_img("common/intel");
                canvas.draw(&*img, drawparams);
            }
        }

        let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x:0.,y:0.,h: 64., w: s.width as f32}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?;
        canvas.draw(&mesh, DrawParam::default());

        for mat in 0..self.level.palette.len() as u8 {
            let x = START_X + f32::from(mat) * 36.;

            if Tool::Inserter(Insertion::Material(mat)) == self.current {
                let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: x - 1., y: 15., w: 34., h: 34.}, YELLOW)?;
                canvas.draw(&mesh, DrawParam::default());
            }
            self.level.palette.draw_mat(mat, canvas, &s.assets, x, 16., DrawParam::default());
        }

        self.entities_bar.draw(ctx, canvas, s, if let Tool::Inserter(ins) = self.current{Some(ins)}else{None})?;
        self.extra_bar.draw(ctx, canvas, s, if let Tool::Inserter(ins) = self.current{Some(ins)}else{None})?;

        self.mat_text.draw_text(canvas);
        self.entities_bar.ent_text.draw_text(canvas);
        self.extra_bar.ent_text.draw_text(canvas);

        Ok(())
    }
    #[allow(clippy::cognitive_complexity)]
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        let shift = ctx.keyboard.is_mod_active(KeyMods::SHIFT);
        let ctrl = ctx.keyboard.is_mod_active(KeyMods::CTRL);

        use self::KeyCode::*;
        match event {
            Key(Z) => self.level.save(&self.save).unwrap(),
            Key(X) => self.level = Level::load(&self.save).unwrap(),
            Key(C) => self.draw_visibility_cones.toggle(),
            Key(G) => self.snap_on_grid.toggle(),
            Key(P) => {
                s.switch(StateSwitch::Play(self.level.clone()));
            }
            Key(T) => self.current = Tool::Selector(Selection::default()),
            Key(Delete) | Key(Back) => if let Tool::Selector(ref mut selection) = self.current {
                #[allow(clippy::unneeded_field_pattern)]
                let Selection {
                    mut enemies,
                    mut waypoints,
                    mut intels,
                    mut pickups,
                    mut weapons,
                    mut decals,
                    exit, moving: _,
                } = ::std::mem::replace(selection, Selection::default());

                if exit {
                    self.level.exit = None;
                }
                waypoints.sort();
                for (e, w) in waypoints.into_iter().rev() {
                    self.level.enemies[e].behaviour.path.remove(w);
                }
                enemies.sort();
                for enemy in enemies.into_iter().rev() {
                    self.level.enemies.remove(enemy);
                }
                intels.sort();
                for intel in intels.into_iter().rev() {
                    self.level.intels.remove(intel);
                }
                decals.sort();
                for decal in decals.into_iter().rev() {
                    self.level.decals.remove(decal);
                }
                pickups.sort();
                for pickup in pickups.into_iter().rev() {
                    self.level.pickups.remove(pickup);
                }
                weapons.sort();
                for weapon in weapons.into_iter().rev() {
                    self.level.weapons.remove(weapon);
                }
            }
            Key(Q) => {
                self.rotation_speed = 0.;
                if shift {
                    match self.current {
                        Tool::Inserter(Insertion::Enemy{ref mut rot}) => *rot -= std::f32::consts::FRAC_PI_4,
                        Tool::Inserter(Insertion::Decal{ref mut rot, ..}) => *rot -= std::f32::consts::FRAC_PI_4,
                        _ => (),
                    }
                }
            }
            Key(E) => {
                self.rotation_speed = 0.;
                if shift {
                    match self.current {
                        Tool::Inserter(Insertion::Enemy{ref mut rot}) => *rot += std::f32::consts::FRAC_PI_4,
                        Tool::Inserter(Insertion::Decal{ref mut rot, ..}) => *rot += std::f32::consts::FRAC_PI_4,
                        _ => (),
                    }
                }
            }
            Key(H) => if let Tool::Selector(Selection { enemies, waypoints, .. }) = &mut self.current {
                match (&mut **enemies, &mut **waypoints) {
                    (&mut [enem], _) | (_, &mut [(enem, _)]) => self.current = Tool::Inserter(Insertion::Waypoint(enem)),
                    (&mut [enem, ..], _) | (_, &mut [(enem, _), ..]) => { enemies.clear(); enemies.push(enem); waypoints.clear() }
                    _ => (),
                }
            }
            Key(O) => if let Tool::Inserter(Insertion::Waypoint(enem)) = self.current {
                self.level.enemies[enem].behaviour.cyclical_path.toggle();
            }
            Key(Up) if ctrl && shift => {
                self.level.grid.stretch_up();
                self.displace(|pos| *pos += vector![0., 32.]);
            }
            Key(Down) if ctrl && shift => {
                self.level.grid.unstretch_up();
                self.displace(|pos| *pos -= vector![0., 32.]);
            }
            Key(Left) if ctrl && shift => {
                self.level.grid.stretch_left();
                self.displace(|pos| *pos += vector![32., 0.]);
            }
            Key(Right) if ctrl && shift => {
                self.level.grid.unstretch_left();
                self.displace(|pos| *pos -= vector![32., 0.]);
            }
            Key(Up) if ctrl => self.level.grid.shorten(),
            Key(Down) if ctrl => self.level.grid.heighten(),
            Key(Left) if ctrl => self.level.grid.thin(),
            Key(Right) if ctrl => self.level.grid.widen(),
            Mouse(Mb::Middle) | Key(Home) => self.level.start_point = Some(self.mousepos(&s)),
            Mouse(Mb::Left) => self.click(s, ctx),
            _ => ()
        }
    }
    fn event_down(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        use self::KeyCode::*;

        let shift = ctx.keyboard.is_mod_active(KeyMods::SHIFT);
        let mousepos = self.mousepos(&s);

        match event {
            Mouse(Mb::Left) => if let Tool::Selector(ref mut selection) = self.current {
                for &i in &selection.enemies {
                    if (self.level.enemies[i].pl.obj.pos - mousepos).norm() <= 16. {
                        return selection.moving = Some(mousepos);
                    }
                }
                for &i in &selection.intels {
                    if (self.level.intels[i] - mousepos).norm() <= 16. {
                        return selection.moving = Some(mousepos);
                    }
                }
                for &i in &selection.decals {
                    if (self.level.decals[i].obj.pos - mousepos).norm() <= 16. {
                        return selection.moving = Some(mousepos);
                    }
                }
                for &i in &selection.pickups {
                    if (self.level.pickups[i].0 - mousepos).norm() <= 16. {
                        return selection.moving = Some(mousepos);
                    }
                }
                for &i in &selection.weapons {
                    if (self.level.weapons[i].pos - mousepos).norm() <= 16. {
                        return selection.moving = Some(mousepos);
                    }
                }
                if selection.exit {
                    if let Some(exit) = self.level.exit {
                        if (exit - mousepos).norm() <= 16. {
                            return selection.moving = Some(mousepos);
                        }
                    }
                }
            }
            Key(Q) if !shift => self.rotation_speed -= 6.,
            Key(E) if !shift => self.rotation_speed += 6.,
            _ => (),
        }
    }
}

impl Editor {
    fn click(&mut self, s: &mut State, ctx: &mut Context) {
        let mousepos = self.mousepos(&s);

        if let Some(ins) = self.extra_bar.click(s.mouse) {
            self.current = Tool::Inserter(ins);
        } else if s.mouse.y <= 64. {
            if s.mouse.x > START_X && s.mouse.x < START_X + self.level.palette.len() as f32 * 36. {
                let i = ((s.mouse.x - START_X) / 36.) as u8;

                self.current = Tool::Inserter(Insertion::Material(i));
            }
            if let Some(ins) = self.entities_bar.click(s.mouse) {
                self.current = Tool::Inserter(ins);
            }
        } else {
            match self.current {
                Tool::Inserter(Insertion::Material(_)) => (),
                Tool::Selector(ref mut selection) => {

                    if let Some(moved_from) = selection.moving {
                        let dist = mousepos - moved_from;

                        if selection.exit {
                            if let Some(ref mut exit) = self.level.exit {
                                *exit += dist;
                            }
                        }
                        for i in selection.enemies.iter().rev() {
                            self.level.enemies[*i].pl.obj.pos += dist;
                        }
                        for i in selection.intels.iter().rev() {
                            self.level.intels[*i] += dist;
                        }
                        for i in selection.decals.iter().rev() {
                            self.level.decals[*i].obj.pos += dist;
                        }
                        for i in selection.pickups.iter().rev() {
                            self.level.pickups[*i].0 += dist;
                        }
                        for i in selection.weapons.iter().rev() {
                            self.level.weapons[*i].pos += dist;
                        }
                        selection.moving = None;
                    } else {
                        if !ctx.keyboard.is_mod_active(KeyMods::CTRL) {
                            *selection = Selection::default();
                        }
                        for (i, enemy) in self.level.enemies.iter().enumerate() {
                            if (enemy.pl.obj.pos - mousepos).norm() <= 16. && !selection.enemies.contains(&i) {
                                selection.enemies.push(i);
                                return
                            }
                        }
                        if let Some(exit) = self.level.exit {
                            if (exit - mousepos).norm() <= 16. && !selection.exit {
                                selection.exit = true;
                                return
                            }
                        }
                        for (i, &intel) in self.level.intels.iter().enumerate() {
                            if (intel - mousepos).norm() <= 16. && !selection.intels.contains(&i) {
                                selection.intels.push(i);
                                return
                            }
                        }
                        for (i, decal) in self.level.decals.iter().enumerate() {
                            if (decal.obj.pos - mousepos).norm() <= 16. && !selection.decals.contains(&i) {
                                selection.decals.push(i);
                                return
                            }
                        }
                        for (i, &pickup) in self.level.pickups.iter().enumerate() {
                            if (pickup.0 - mousepos).norm() <= 16. && !selection.pickups.contains(&i) {
                                selection.pickups.push(i);
                                return
                            }
                        }
                        for (i, weapon) in self.level.weapons.iter().enumerate() {
                            if (weapon.pos - mousepos).norm() <= 16. && !selection.weapons.contains(&i) {
                                selection.weapons.push(i);
                                return
                            }
                        }
                    }
                }
                Tool::Inserter(Insertion::Exit) => {
                    self.level.exit = Some(self.mousepos(&s));
                    self.current = Tool::Selector(Selection{exit: true, .. Default::default()});
                }
                Tool::Inserter(Insertion::Enemy{rot}) => {
                    s.mplayer.play(ctx, "reload").unwrap();
                    self.level.enemies.push(Enemy::new(Object::with_rot(mousepos, rot)));
                },
                Tool::Inserter(Insertion::Waypoint(e)) => {
                    self.level.enemies[e].behaviour.path.push(mousepos);
                }
                Tool::Inserter(Insertion::Decal{spr, rot}) => {
                    self.level.decals.push(Decal::new(Object::with_rot(mousepos, rot), spr));
                }
                Tool::Inserter(Insertion::Pickup(i)) => {
                    self.level.pickups.push((mousepos, i));
                },
                Tool::Inserter(Insertion::Weapon(id)) => {
                    self.level.weapons.push(WEAPONS[id].make_drop(mousepos));
                },
                Tool::Inserter(Insertion::Intel) => self.level.intels.push(mousepos),
            }
        }
    }
}