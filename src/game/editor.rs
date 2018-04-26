use ::*;
use graphics::{Rect, DrawMode};
use ggez::error::GameError;
use super::world::*;

use std::path::PathBuf;

use io::snd::Sound;

#[derive(Debug, PartialEq, Clone)]
enum Tool {
    Inserter(Insertion),
    Selector(Selection),
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Insertion {
    Material(Material),
    Intel,
    Enemy{rot: f32},
    Exit,
}

#[derive(Default, Debug, PartialEq, Clone)]
struct Selection {
    exit: bool,
    enemies: Vec<usize>,
    intels: Vec<usize>,
    moving: Option<Point2>,
}

/// The state of the game
pub struct Editor {
    save: PathBuf,
    pos: Point2,
    level: Level,
    current: Tool,
    mat_text: PosText,
    ent_text: PosText,
    draw_visibility_cones: bool,
    rotation_speed: f32,
    snap_on_grid: bool,
}

const PALETTE: [Material; 7] = [
    Material::Grass,
    Material::Dirt,
    Material::Floor,
    Material::Wall,
    Material::Asphalt,
    Material::Sand,
    Material::Concrete,
];

impl Editor {
    pub fn new(ctx: &mut Context, s: &State) -> GameResult<Box<GameState>> {
        let mat_text = s.assets.text(ctx, Point2::new(2., 18.0), "Materials:")?;
        let ent_text = s.assets.text(ctx, Point2::new(302., 18.0), "Entities:")?;

        let save;
        if let Content::File(ref f) = s.content {
            save = f.clone();
        } else {
            return Err(GameError::UnknownError("Cannot load editor in campaign".to_owned()));
        }

        let level = s.level.clone().unwrap_or_else(|| {
            Level::load(&save).unwrap_or_else(|_| Level::new(32, 32))
        });

        let x = level.grid.width() as f32 * 16.;
        let y = level.grid.height() as f32 * 16.;

        Ok(Box::new(Editor {
            save,
            pos: Point2::new(x, y),
            current: Tool::Selector(Selection::default()),
            draw_visibility_cones: false,
            mat_text,
            ent_text,
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
    fn update(&mut self, s: &mut State, _ctx: &mut Context) -> GameResult<()> {
        let speed = match s.modifiers.shift {
            false => 175.,
            true => 315.,
        };
        let v = speed * Vector2::new(s.input.hor(), s.input.ver());
        self.pos += v * DELTA;

        if let Tool::Inserter(Insertion::Enemy{ref mut rot}) = self.current {
            *rot += self.rotation_speed * DELTA;
        }
        Ok(())
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) -> GameResult<()> {
        if s.mouse_down.left && s.mouse.y > 64. {
            if let Tool::Inserter(Insertion::Material(mat)) = self.current {
                let (mx, my) = Grid::snap(s.mouse - s.offset);
                self.level.grid.insert(mx, my, mat);
            }
        }

        s.focus_on(self.pos);
        Ok(())
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.level.grid.draw(ctx, &s.assets)?;

        if let Tool::Inserter(Insertion::Material(mat)) = self.current {
            let (x, y) = Grid::snap(s.mouse-s.offset);
            let x = x as f32 * 32.;
            let y = y as f32 * 32.;
            graphics::set_color(ctx, TRANS)?;
            mat.draw(ctx, &s.assets, x, y)?;
        }

        if let Some(start) = self.level.start_point {
            graphics::set_color(ctx, GREEN)?;
            graphics::circle(ctx, DrawMode::Fill, start, 16., 1.)?;
            graphics::set_color(ctx, BLUE)?;
            graphics::circle(ctx, DrawMode::Fill, start, 9., 1.)?;
        }
        if let Some(exit) = self.level.exit {
            if let Tool::Selector(Selection{exit: true, ..}) = self.current {
                graphics::set_color(ctx, YELLOW)?;
                graphics::circle(ctx, DrawMode::Fill, exit, 17., 0.5)?;
            }
            let drawparams = graphics::DrawParam {
                dest: exit,
                offset: Point2::new(0.5, 0.5),
                color: Some(graphics::WHITE),
                .. Default::default()
            };
            graphics::draw_ex(ctx, s.assets.get_img(Sprite::Goal), drawparams)?;
        }

        for (i, &intel) in self.level.intels.iter().enumerate() {
            if let Tool::Selector(Selection{ref intels, ..}) = self.current {
                if intels.contains(&i) {
                    graphics::set_color(ctx, YELLOW)?;
                    graphics::circle(ctx, DrawMode::Fill, intel, 17., 0.5)?;
                }
            }
            let drawparams = graphics::DrawParam {
                dest: intel,
                offset: Point2::new(0.5, 0.5),
                color: Some(graphics::WHITE),
                .. Default::default()
            };
            graphics::draw_ex(ctx, s.assets.get_img(Sprite::Intel), drawparams)?;
        }

        for (i, enemy) in self.level.enemies.iter().enumerate() {
            if let Tool::Selector(Selection{ref enemies, ..})= self.current {
                if enemies.contains(&i) {
                    graphics::set_color(ctx, YELLOW)?;
                    graphics::circle(ctx, DrawMode::Fill, enemy.obj.pos, 17., 0.5)?;
                }
            }
            if self.draw_visibility_cones {
                graphics::set_color(ctx, BLUE)?;
                enemy.draw_visibility_cone(ctx, 512.)?;
            }
            graphics::set_color(ctx, graphics::WHITE)?;
            enemy.draw(ctx, &s.assets)?;
        }

        if let Tool::Selector(ref selection @ Selection{moving: Some(_), ..}) = self.current {
            let mousepos = self.mousepos(s);
            let dist = mousepos - selection.moving.unwrap();

            graphics::set_color(ctx, TRANS)?;
            for &i in &selection.enemies {
                let mut enem = self.level.enemies[i].clone();
                enem.obj.pos += dist;
                enem.draw(ctx, &s.assets)?;
            }
            for &i in &selection.intels {
                let drawparams = graphics::DrawParam {
                    dest: self.level.intels[i] + dist,
                    offset: Point2::new(0.5, 0.5),
                    .. Default::default()
                };
                graphics::draw_ex(ctx, s.assets.get_img(Sprite::Intel), drawparams)?;
            }
            if selection.exit {
                if let Some(exit) = self.level.exit {
                    let drawparams = graphics::DrawParam {
                        dest: exit + dist,
                        offset: Point2::new(0.5, 0.5),
                        .. Default::default()
                    };
                    graphics::draw_ex(ctx, s.assets.get_img(Sprite::Goal), drawparams)?;
                }
            }
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        let dest = self.mousepos(s) + s.offset;
        match self.current {
            Tool::Selector(_) => (),
            Tool::Inserter(Insertion::Material(_)) => (),
            Tool::Inserter(Insertion::Enemy{rot}) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    rotation: rot,
                    offset: Point2::new(0.5, 0.5),
                    color: Some(TRANS),
                    .. Default::default()
                };
                graphics::draw_ex(ctx, s.assets.get_img(Sprite::Enemy), drawparams)?;
            }
            Tool::Inserter(Insertion::Exit) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    offset: Point2::new(0.5, 0.5),
                    color: Some(TRANS),
                    .. Default::default()
                };
                graphics::draw_ex(ctx, s.assets.get_img(Sprite::Goal), drawparams)?;
            }
            Tool::Inserter(Insertion::Intel) => {
                let drawparams = graphics::DrawParam {
                    dest,
                    offset: Point2::new(0.5, 0.5),
                    color: Some(TRANS),
                    .. Default::default()
                };
                graphics::draw_ex(ctx, s.assets.get_img(Sprite::Intel), drawparams)?;
            }
        }

        graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x:0.,y:0.,h: 64., w: s.width as f32})?;
        graphics::set_color(ctx, graphics::WHITE)?;

        for (i, mat) in PALETTE.iter().enumerate() {
            let x = START_X + i as f32 * 36.;
            if Tool::Inserter(Insertion::Material(*mat)) == self.current {
                graphics::set_color(ctx, YELLOW)?;
                graphics::rectangle(ctx, DrawMode::Fill, Rect{x: x - 1., y: 15., w: 34., h: 34.})?;
                graphics::set_color(ctx, graphics::WHITE)?;
            }
            mat.draw(ctx, &s.assets, x, 16.)?;
        }

        if let Tool::Inserter(Insertion::Enemy{..}) = self.current {
            graphics::set_color(ctx, YELLOW)?;
            graphics::circle(ctx, DrawMode::Fill, Point2::new(400., 34.), 17., 0.5)?;
        }
        let drawparams = graphics::DrawParam {
            dest: Point2::new(400., 34.),
            offset: Point2::new(0.5, 0.5),
            color: Some(graphics::WHITE),
            .. Default::default()
        };
        graphics::draw_ex(ctx, s.assets.get_img(Sprite::Enemy), drawparams)?;

        if let Tool::Inserter(Insertion::Exit) = self.current {
            graphics::set_color(ctx, YELLOW)?;
            graphics::circle(ctx, DrawMode::Fill, Point2::new(434., 34.), 17., 0.5)?;
        }
        let drawparams = graphics::DrawParam {
            dest: Point2::new(434., 34.),
            offset: Point2::new(0.5, 0.5),
            color: Some(graphics::WHITE),
            .. Default::default()
        };
        graphics::draw_ex(ctx, s.assets.get_img(Sprite::Goal), drawparams)?;

        if let Tool::Inserter(Insertion::Intel) = self.current {
            graphics::set_color(ctx, YELLOW)?;
            graphics::circle(ctx, DrawMode::Fill, Point2::new(468., 34.), 17., 0.5)?;
        }
        let drawparams = graphics::DrawParam {
            dest: Point2::new(468., 34.),
            offset: Point2::new(0.5, 0.5),
            color: Some(graphics::WHITE),
            .. Default::default()
        };
        graphics::draw_ex(ctx, s.assets.get_img(Sprite::Intel), drawparams)?;

        graphics::set_color(ctx, graphics::WHITE)?;
        self.mat_text.draw_text(ctx)?;
        self.ent_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            Z => self.level.save(&self.save).unwrap(),
            X => self.level = Level::load(&self.save).unwrap(),
            C => self.draw_visibility_cones.toggle(),
            G => self.snap_on_grid.toggle(),
            P => {
                s.level = Some(self.level.clone());
                s.switch(StateSwitch::Play)
            }
            T => self.current = Tool::Selector(Selection::default()),
            Delete | Backspace => if let Tool::Selector(ref mut selection) = self.current {
                let Selection {
                    mut enemies,
                    mut intels,
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
            }
            Comma => self.rotation_speed += 6.,
            Period => self.rotation_speed -= 6.,
            Up if s.modifiers.ctrl => self.level.grid.shorten(),
            Down if s.modifiers.ctrl => self.level.grid.heighten(),
            Left if s.modifiers.ctrl => self.level.grid.thin(),
            Right if s.modifiers.ctrl => self.level.grid.widen(),
            _ => return,
        }
    }
    fn mouse_down(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use MouseButton::*;
        let mousepos = self.mousepos(&s);
        match btn {
            Left => if let Tool::Selector(ref mut selection) = self.current {
                for &i in &selection.enemies {
                    if (self.level.enemies[i].obj.pos - mousepos).norm() <= 16. {
                        return selection.moving = Some(mousepos);
                    }
                }
                for &i in &selection.intels {
                    if (self.level.intels[i] - mousepos).norm() <= 16. {
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
            _ => ()
        }
    }
    fn mouse_up(&mut self, s: &mut State, ctx: &mut Context, btn: MouseButton) {
        use MouseButton::*;
        let mousepos = self.mousepos(&s);
        match btn {
            Left => if s.mouse.y <= 64. {
                if s.mouse.x > START_X && s.mouse.x < START_X + PALETTE.len() as f32 * 36. {
                    let i = ((s.mouse.x - START_X) / 36.) as usize;

                    self.current = Tool::Inserter(Insertion::Material(PALETTE[i]));
                }
                if s.mouse.y >= 18. && s.mouse.y < 50. {
                    if s.mouse.x >= 384. && s.mouse.x < 416. {
                        self.current = Tool::Inserter(Insertion::Enemy{rot: 0.});
                    }
                    if s.mouse.x >= 418. && s.mouse.x < 450. {
                        self.current = Tool::Inserter(Insertion::Exit);
                    }
                    if s.mouse.x >= 452. && s.mouse.x < 484. {
                        self.current = Tool::Inserter(Insertion::Intel);
                    }
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
                                self.level.enemies[*i].obj.pos += dist;
                            }
                            for i in selection.intels.iter().rev() {
                                self.level.intels[*i] += dist;
                            }
                            selection.moving = None;
                        } else {
                            for (i, enemy) in self.level.enemies.iter().enumerate() {
                                if (enemy.obj.pos - mousepos).norm() <= 16. {
                                    if !selection.enemies.contains(&i) {
                                        if s.modifiers.ctrl {
                                            selection.enemies.push(i);
                                        } else {
                                            *selection = Selection{enemies: vec![i], .. Default::default()};
                                        }
                                        return
                                    }
                                }
                            }
                            if let Some(exit) = self.level.exit {
                                if (exit - mousepos).norm() <= 16. {
                                    if !selection.exit {
                                        if s.modifiers.ctrl {
                                            selection.exit = true;
                                        } else {
                                            *selection = Selection{exit: true, .. Default::default()};
                                        }
                                        return
                                    }
                                }
                            }
                            for (i, &intel) in self.level.intels.iter().enumerate() {
                                if (intel - mousepos).norm() <= 16. {
                                    if !selection.intels.contains(&i) {
                                        if s.modifiers.ctrl {
                                            selection.intels.push(i);
                                        } else {
                                            *selection = Selection{intels: vec![i], .. Default::default()};
                                        }
                                        return
                                    }
                                }
                            }
                            *selection = Selection::default();
                        }
                    }
                    Tool::Inserter(Insertion::Exit) => {
                        self.level.exit = Some(self.mousepos(&s));
                        self.current = Tool::Selector(Selection{exit: true, .. Default::default()});
                    }
                    Tool::Inserter(Insertion::Enemy{rot}) => {
                        s.mplayer.play(ctx, Sound::Reload).unwrap();
                        self.level.enemies.push(Enemy::new(Object::with_rot(mousepos, rot)));
                    },
                    Tool::Inserter(Insertion::Intel) => self.level.intels.push(mousepos),
                }
            }
            Middle => self.level.start_point = Some(self.mousepos(&s)),
            _ => ()
        }
    }
    fn key_down(&mut self, s: &mut State,_ctx: &mut Context,  keycode: Keycode) {
        use Keycode::*;
        match keycode {
            Comma => self.rotation_speed -= 6.,
            Period => self.rotation_speed += 6.,
            Q => self.level.start_point = Some(self.mousepos(&s)),
            _ => return,
        }
    }
}
