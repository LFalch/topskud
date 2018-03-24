use ::*;
use graphics::{Rect, DrawMode};
use io::snd::Sound;

/// The state of the game
pub struct Menu {
    play_text: PosText,
    editor_text: PosText,
    cur_lvl_text: Option<PosText>,
}

impl Menu {
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<GameState>> {
        let w = ctx.conf.window_mode.width as f32 / 2.;

        let play_text = s.assets.text(ctx, Point2::new(w-2.*10., 80.), "Play")?;
        let editor_text = s.assets.text(ctx, Point2::new(w-3.*10., 146.), "Editor")?;
        let cur_lvl_text = if let Content::File(ref f) = s.content {
            Some(s.assets.text(ctx, Point2::new(2., 2.0), &format!("Current level: {}", f.display()))?)
        } else {
            None
        };
        s.mplayer.play(ctx, Sound::Music)?;

        Ok(Box::new(Menu {
            play_text,
            editor_text,
            cur_lvl_text,
        }))
    }
    pub fn switch_play(&self, ctx: &mut Context, s: &mut State) {
        s.mplayer.stop(ctx, Sound::Music).unwrap();
        s.switch(StateSwitch::Play);
    }
    pub fn switch_editor(&self, ctx: &mut Context, s: &mut State) {
        s.mplayer.stop(ctx, Sound::Music).unwrap();
        s.switch(StateSwitch::Editor);
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
        if let Some(ref txt) = self.cur_lvl_text {
            txt.draw_text(ctx)?;
        }
        self.play_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, ctx: &mut Context, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            P => self.switch_play(ctx, s),
            E => self.switch_editor(ctx, s),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, ctx: &mut Context, btn: MouseButton) {
        let w = s.width as f32;
        let x = 3. * w / 7.;
        let w = w / 7.;
        use MouseButton::*;
        match btn {
            Left => {
                if s.mouse.x >= x && s.mouse.x < x + w{
                    if s.mouse.y >= PLAY_Y && s.mouse.y < PLAY_Y + 64. {
                        self.switch_play(ctx, s);
                    }
                    if s.mouse.y >= EDITOR_Y && s.mouse.y < EDITOR_Y + 64. {
                        self.switch_editor(ctx, s);
                    }
                }
            }
            _ => ()
        }
    }
}
