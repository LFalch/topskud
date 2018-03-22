use ::*;
use graphics::{Rect, DrawMode};
use super::world::*;

use std::path::PathBuf;

/// The state of the game
pub struct Menu {
    save: PathBuf,
    dims: Option<(usize, usize)>,
    play_text: PosText,
    editor_text: PosText,
    cur_lvl_text: PosText,
}

impl Menu {
    pub fn new(ctx: &mut Context, s: &State, save: PathBuf, dims: Option<(usize, usize)>) -> GameResult<Box<GameState>> {
        let w = ctx.conf.window_mode.width as f32 / 2.;

        let play_text = s.assets.text(ctx, Point2::new(w-2.*10., 80.), "Play")?;
        let editor_text = s.assets.text(ctx, Point2::new(w-3.*10., 146.), "Editor")?;
        let cur_lvl_text = s.assets.text(ctx, Point2::new(2., 2.0), &format!("Current level: {}", save.display()))?;

        Ok(Box::new(Menu {
            save,
            dims,
            play_text,
            editor_text,
            cur_lvl_text,
        }))
    }
    pub fn switch_play(&self, s: &mut State) {
        if self.dims.is_none() {
            let level = Level::load(&self.save).unwrap_or_else(|_| Level::new(1, 1));
            s.switch(StateSwitch::Play(level));
        }
    }
    pub fn switch_editor(&self, s: &mut State) {
        s.switch(StateSwitch::Editor {
            save: self.save.clone(),
            dims: self.dims,
        });
    }
    pub fn rect(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) -> GameResult<()> {
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x, y, h, w})
    }
}

const PLAY_Y: f32 = 64.;
const EDITOR_Y: f32 = 132.;

impl GameState for Menu {
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        let w = s.width as f32;
        let x = 3. * w / 7.;
        let w = w / 7.;
        graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.75, a: 1.})?;
        Menu::rect(ctx, x, PLAY_Y, w, 64.)?;
        Menu::rect(ctx, x, EDITOR_Y, w, 64.)?;

        graphics::set_color(ctx, graphics::WHITE)?;
        self.editor_text.draw_text(ctx)?;
        self.cur_lvl_text.draw_text(ctx)?;

        if self.dims.is_some() {
            graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?;
        }

        self.play_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            P => self.switch_play(s),
            E => self.switch_editor(s),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        let w = s.width as f32;
        let x = 3. * w / 7.;
        let w = w / 7.;
        use MouseButton::*;
        match btn {
            Left => {
                if s.mouse.x >= x && s.mouse.x < x + w{
                    if s.mouse.y >= PLAY_Y && s.mouse.y < PLAY_Y + 64. {
                        self.switch_play(s);
                    }
                    if s.mouse.y >= EDITOR_Y && s.mouse.y < EDITOR_Y + 64. {
                        self.switch_editor(s);
                    }
                }
            }
            _ => ()
        }
    }
}
