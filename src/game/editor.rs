use ::*;
use graphics::{Rect, DrawMode};
use super::world::*;
use super::play::Play;

use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Tool {
    Material(Material),
    Selector,
    Enemy,
}

/// The state of the game
pub struct Editor {
    pos: Point2,
    fast: bool,
    level: Level,
    current: Tool,
    mat_text: PosText,
    ent_text: PosText,
    save: PathBuf,
}

const PALETTE: [Material; 5] = [
    Material::Grass,
    Material::Dirt,
    Material::Floor,
    Material::Wall,
    Material::Concrete,
];

impl Editor {
    pub fn new<P: AsRef<Path>>(ctx: &mut Context, assets: &Assets, save: P, dims: Option<(usize, usize)>) -> GameResult<Self> {
        // Initialise the text objects
        let mat_text = assets.text(ctx, Point2::new(2., 18.0), "Materials:")?;
        let ent_text = assets.text(ctx, Point2::new(302., 18.0), "Entities:")?;
        let (x, y);
        let level = if let Some((w, h)) = dims {
            x = w as f32 * 16.;
            y = h as f32 * 16.;
            Level::new(w, h)
        } else {
            x = 16. * 32.;
            y = 16. * 32.;
            Level::load(save.as_ref()).unwrap_or_else(|_| Level::new(32, 32))
        };

        Ok(Editor {
            fast: false,
            pos: Point2::new(x, y),
            current: Tool::Material(Material::Wall),
            mat_text,
            ent_text,
            level,
            save: save.as_ref().to_path_buf(),
        })
    }
}
const START_X: f32 = 103.;

impl GameState for Editor {
    fn update(&mut self, s: &mut State) {
        let speed = match self.fast {
            false => 175.,
            true => 315.,
        };
        let v = speed * Vector2::new(s.input.hor(), s.input.ver());
        self.pos += v * DELTA;
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) {
        if s.mouse_down.left && s.mouse.y > 64. {
            if let Tool::Material(mat) = self.current {
                let (mx, my) = Grid::snap(s.mouse - s.offset);
                self.level.grid.insert(mx, my, mat);
            }
        }

        s.focus_on(self.pos);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.level.grid.draw(ctx, &s.assets)?;
        if let Some(start) = self.level.start_point {
            graphics::set_color(ctx, GREEN)?;
            graphics::circle(ctx, DrawMode::Fill, start, 16., 1.)?;
            graphics::set_color(ctx, BLUE)?;
            graphics::circle(ctx, DrawMode::Fill, start, 9., 1.)?;
        }
        for enemy in &self.level.enemies {
            enemy.draw(ctx, &s.assets)?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x:0.,y:0.,h: 64., w: s.width as f32})?;
        graphics::set_color(ctx, graphics::WHITE)?;

        for (i, mat) in PALETTE.iter().enumerate() {
            let x = START_X + i as f32 * 36.;
            if Tool::Material(*mat) == self.current {
                graphics::set_color(ctx, Color{r: 1., g: 1., b: 0., a: 1.})?;
                graphics::rectangle(ctx, DrawMode::Fill, Rect{x: x - 1., y: 15., w: 34., h: 34.})?;
                graphics::set_color(ctx, graphics::WHITE)?;
            }
            mat.draw(ctx, &s.assets, x, 16.)?;
        }

        self.mat_text.draw_text(ctx)?;
        self.ent_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            Z => self.level.save(&self.save).unwrap(),
            X => self.level = Level::load(&self.save).unwrap(),
            P => s.switch(Box::new(Play::new(self.level.clone()))),
            LShift => self.fast = false,
            _ => return,
        }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use MouseButton::*;
        match btn {
            Left if s.mouse.y <= 64. => {
                if s.mouse.x > START_X && s.mouse.x < START_X + PALETTE.len() as f32 * 36. {
                    let i = ((s.mouse.x - START_X) / 36.) as usize;

                    self.current = Tool::Material(PALETTE[i]);
                }
            }
            Right => self.level.start_point = Some(s.mouse - s.offset),
            Middle => self.level.enemies.push(Enemy::new(Object::new(s.mouse - s.offset))),
            _ => ()
        }
    }
    fn key_down(&mut self, _s: &mut State,_ctx: &mut Context,  keycode: Keycode) {
        use Keycode::*;
        match keycode {
            LShift => self.fast = true,
            _ => return,
        }
    }
}
