use crate::{
    util::{
        ver,
        hor,
        sstr,
        TRANS,
        Vector2, Point2},
    io::tex::PosText,
    ext::BoolExt,
    game::{
        DELTA, Content, GameState, State, StateSwitch,
        world::{Grid, Level, Palette},
        event::{Event::{self, Key, Mouse}, MouseButton as Mb, KeyCode, KeyMods}
    },
    obj::{Object, enemy::Enemy, decal::Decal, pickup::PICKUPS, weapon::WEAPONS}
};
use ggez::{
    Context, GameResult,
    graphics::{self, Color, WHITE, Rect, DrawMode, DrawParam, Mesh},
    error::GameError,
    input::{
        keyboard,
        mouse,
    },
};

use std::path::PathBuf;
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
    fn draw(&self, ctx: &mut Context, s: &State, cur: Option<Insertion>) -> GameResult<()> {
        let mut drawparams = graphics::DrawParam {
            dest: (self.ent_text.pos + Vector2::new(98., 16.)).into(),
            offset: Point2::new(0.5, 0.5).into(),
            .. Default::default()
        };

        for ins in &*self.palette {
            if let Some(cur) = cur {
                if ins == &cur {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), drawparams.dest, 17., 0.5, YELLOW)?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
            let img = s.assets.get_img(ctx, ins.get_spr());
            graphics::draw(ctx, &*img, drawparams)?;
            drawparams.dest.x += 34.; 
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
        let mat_text = s.assets.text(Point2::new(2., 18.0)).and_text("Materials:");
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

        let entities_bar = InsertionBar::new(Point2::new(392., 18.0), s, "Entitites:", entities.into_boxed_slice());
        let extra_bar = InsertionBar::new(Point2::new(392., 52.0), s, "", extra_entities);

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
            pos: Point2::new(x, y),
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
}

const START_X: f32 = 103.;
const YELLOW: Color = Color{r: 1., g: 1., b: 0., a: 1.};

impl GameState for Editor {
    fn update(&mut self, _s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let speed = if keyboard::is_mod_active(ctx, KeyMods::SHIFT) { 315. } else { 175. };
        let v = speed * Vector2::new(hor(ctx), ver(ctx));
        self.pos += v * DELTA;

        match self.current {
            Tool::Inserter(Insertion::Enemy{ref mut rot}) => *rot += self.rotation_speed * DELTA,
            Tool::Inserter(Insertion::Decal{ref mut rot, ..}) => *rot += self.rotation_speed * DELTA,
            _ => (),
        }
        Ok(())
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        if mouse::button_pressed(ctx, Mb::Left) && s.mouse.y > 64. {
            if let Tool::Inserter(Insertion::Material(mat)) = self.current {
                let (mx, my) = Grid::snap(s.mouse - s.offset);
                self.level.grid.insert(mx, my, mat);
            }
        }

        s.focus_on(self.pos);
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.level.grid.draw(&self.level.palette, ctx, &s.assets)?;

        if let Tool::Inserter(Insertion::Material(mat)) = self.current {
            let (x, y) = Grid::snap(s.mouse-s.offset);
            let x = f32::from(x) * 32.;
            let y = f32::from(y) * 32.;
            self.level.palette.draw_mat(mat, ctx, &s.assets, x, y, graphics::DrawParam {
                color: TRANS,
                .. Default::default()
            })?;
        }

        if let Some(start) = self.level.start_point {
            let img = s.assets.get_img(ctx, "common/start");
            graphics::draw(ctx, &*img, graphics::DrawParam {
                dest: start.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            })?;
        }
        if let Some(exit) = self.level.exit {
            if let Tool::Selector(Selection{exit: true, ..}) = self.current {
                let mesh = Mesh::new_circle(ctx, DrawMode::fill(), exit, 17., 0.5, YELLOW)?;
                graphics::draw(ctx, &mesh, DrawParam::default())?;
            }
            let drawparams = graphics::DrawParam {
                dest: exit.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, "common/goal");
            graphics::draw(ctx, &*img, drawparams)?;
        }

        for (i, &intel) in self.level.intels.iter().enumerate() {
            if let Tool::Selector(Selection{ref intels, ..}) = self.current {
                if intels.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), intel, 17., 0.5, YELLOW)?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
            let drawparams = graphics::DrawParam {
                dest: intel.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, "common/intel");
            graphics::draw(ctx, &*img, drawparams)?;
        }

        for (i, enemy) in self.level.enemies.iter().enumerate() {
            if let Tool::Selector(Selection{ref enemies, ..})= self.current {
                if enemies.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), enemy.pl.obj.pos, 17., 0.5, YELLOW)?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
            if self.draw_visibility_cones {
                enemy.draw_visibility_cone(ctx, 512.)?;
            }
            enemy.draw(ctx, &s.assets, WHITE)?;
        }
        for (i, decal) in self.level.decals.iter().enumerate() {
            if let Tool::Selector(Selection{ref decals, ..})= self.current {
                if decals.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), decal.obj.pos, 17., 0.5, YELLOW)?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
            decal.draw(ctx, &s.assets, WHITE)?;
        }

        // Draw init pick-up-ables on top of enemies so they're visible
        for (i, pickup) in self.level.pickups.iter().enumerate() {
            if let Tool::Selector(Selection{ref pickups, ..}) = self.current {
                if pickups.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), pickup.0, 17., 0.5, YELLOW)?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
            PICKUPS[pickup.1 as usize].draw(pickup.0, ctx, &s.assets)?;
        }
        for (i, weapon) in self.level.weapons.iter().enumerate() {
            if let Tool::Selector(Selection{ref weapons, ..}) = self.current {
                if weapons.contains(&i) {
                    let mesh = Mesh::new_circle(ctx, DrawMode::fill(), weapon.pos, 17., 0.5, YELLOW)?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
            let drawparams = graphics::DrawParam {
                dest: weapon.pos.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, &weapon.weapon.entity_sprite);
            graphics::draw(ctx, &*img, drawparams)?;
        }

        // Draw moving objects shadows
        if let Tool::Selector(ref selection @ Selection{moving: Some(_), ..}) = self.current {
            let mousepos = self.mousepos(s);
            let dist = mousepos - selection.moving.unwrap();

            for &i in &selection.enemies {
                let mut enem = self.level.enemies[i].clone();
                enem.pl.obj.pos += dist;
                enem.draw(ctx, &s.assets, TRANS)?;
            }
            for &i in &selection.intels {
                let drawparams = graphics::DrawParam {
                    dest: (self.level.intels[i] + dist).into(),
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, "common/intel");
                graphics::draw(ctx, &*img, drawparams)?;
            }
            for &i in &selection.decals {
                let mut dec = self.level.decals[i].clone();
                dec.obj.pos += dist;
                dec.draw(ctx, &s.assets, TRANS)?;
            }
            for &i in &selection.pickups {
                let pickup = self.level.pickups[i];
                let drawparams = graphics::DrawParam {
                    dest: (pickup.0 + dist).into(),
                    offset: (Point2::new(0.5, 0.5)).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, PICKUPS[pickup.1 as usize].spr);
                graphics::draw(ctx, &*img, drawparams)?;
            }
            for &i in &selection.weapons {
                let drawparams = graphics::DrawParam {
                    dest: (self.level.weapons[i].pos + dist).into(),
                    offset: (Point2::new(0.5, 0.5)).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, &self.level.weapons[i].weapon.entity_sprite);
                graphics::draw(ctx, &*img, drawparams)?;
            }
            if selection.exit {
                if let Some(exit) = self.level.exit {
                    let drawparams = graphics::DrawParam {
                        dest: (exit + dist).into(),
                        offset: (Point2::new(0.5, 0.5)).into(),
                        color: TRANS,
                        .. Default::default()
                    };
                    let img = s.assets.get_img(ctx, "common/goal");
            graphics::draw(ctx, &*img, drawparams)?;
                }
            }
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        let dest = (self.mousepos(s) + s.offset).into();
        match self.current {
            Tool::Selector(_) => (),
            Tool::Inserter(Insertion::Material(_)) => (),
            Tool::Inserter(Insertion::Pickup(index)) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    rotation: 0.,
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, PICKUPS[index as usize].spr);
                graphics::draw(ctx, &*img, drawparams)?;
            }
            Tool::Inserter(Insertion::Weapon(id)) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    rotation: 0.,
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, &WEAPONS[id].entity_sprite);
                graphics::draw(ctx, &*img, drawparams)?;
            }
            Tool::Inserter(Insertion::Enemy{rot}) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    rotation: rot,
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, "common/enemy");
                graphics::draw(ctx, &*img, drawparams)?;
            }
            Tool::Inserter(Insertion::Decal{spr, rot}) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    rotation: rot,
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, spr);
                graphics::draw(ctx, &*img, drawparams)?;
            }
            Tool::Inserter(Insertion::Exit) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, "common/goal");
                graphics::draw(ctx, &*img, drawparams)?;
            }
            Tool::Inserter(Insertion::Intel) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    offset: Point2::new(0.5, 0.5).into(),
                    color: TRANS,
                    .. Default::default()
                };
                let img = s.assets.get_img(ctx, "common/intel");
                graphics::draw(ctx, &*img, drawparams)?;
            }
        }

        let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x:0.,y:0.,h: 64., w: s.width as f32}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        for mat in 0..self.level.palette.len() as u8 {
            let x = START_X + f32::from(mat) * 36.;

            if Tool::Inserter(Insertion::Material(mat)) == self.current {
                let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: x - 1., y: 15., w: 34., h: 34.}, YELLOW)?;
                graphics::draw(ctx, &mesh, DrawParam::default())?;
            }
            self.level.palette.draw_mat(mat, ctx, &s.assets, x, 16., DrawParam::default())?;
        }

        self.entities_bar.draw(ctx, s, if let Tool::Inserter(ins) = self.current{Some(ins)}else{None})?;
        self.extra_bar.draw(ctx, s, if let Tool::Inserter(ins) = self.current{Some(ins)}else{None})?;

        self.mat_text.draw_text(ctx)?;
        self.entities_bar.ent_text.draw_text(ctx)?;
        self.extra_bar.ent_text.draw_text(ctx)
    }
    #[allow(clippy::cognitive_complexity)]
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        let shift = keyboard::is_mod_active(ctx, KeyMods::SHIFT);
        let ctrl = keyboard::is_mod_active(ctx, KeyMods::CTRL);

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
                    mut intels,
                    mut pickups,
                    mut weapons,
                    mut decals,
                    exit, moving: _,
                } = ::std::mem::replace(selection, Selection::default());

                if exit {
                    self.level.exit = None;
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
            Key(Comma) => {
                self.rotation_speed = 0.;
                if shift {
                    match self.current {
                        Tool::Inserter(Insertion::Enemy{ref mut rot}) => *rot -= std::f32::consts::FRAC_PI_4,
                        Tool::Inserter(Insertion::Decal{ref mut rot, ..}) => *rot -= std::f32::consts::FRAC_PI_4,
                        _ => (),
                    }
                }
            }
            Key(Period) => {
                self.rotation_speed = 0.;
                if shift {
                    match self.current {
                        Tool::Inserter(Insertion::Enemy{ref mut rot}) => *rot += std::f32::consts::FRAC_PI_4,
                        Tool::Inserter(Insertion::Decal{ref mut rot, ..}) => *rot += std::f32::consts::FRAC_PI_4,
                        _ => (),
                    }
                }
            }
            Key(Up) if ctrl => self.level.grid.shorten(),
            Key(Down) if ctrl => self.level.grid.heighten(),
            Key(Left) if ctrl => self.level.grid.thin(),
            Key(Right) if ctrl => self.level.grid.widen(),
            Mouse(Mb::Middle) | Key(Q) => self.level.start_point = Some(self.mousepos(&s)),
            Mouse(Mb::Left) => self.click(s, ctx),
            _ => (),
        }
    }
    fn event_down(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        use self::KeyCode::*;

        let shift = keyboard::is_mod_active(ctx, KeyMods::SHIFT);
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
            Key(Comma) if !shift => self.rotation_speed -= 6.,
            Key(Period) if !shift => self.rotation_speed += 6.,
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
                        if !keyboard::is_mod_active(ctx, KeyMods::CTRL) {
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
                    self.level.weapons.push(WEAPONS["glock"].make_drop(mousepos));
                },
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