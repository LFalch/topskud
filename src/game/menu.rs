use crate::*;
use crate::graphics::Rect;
use crate::io::snd::Sound;
use crate::io::btn::Button;

/// The state of the game
pub struct Menu {
    title_txt: PosText,
    play_btn: Button,
    editor_btn: Button,
    cur_lvl_text: Option<PosText>,
}

// ↓
fn button_rect(w: f32, i: f32) -> Rect {
    Rect{x:3. * w / 7., y: 64. + i * 68., w:w / 7., h:64.}
}

impl Menu {
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<GameState>> {
        let w = s.width as f32;

        let cur_lvl_text = if let Content::File(ref f) = s.content {
            Some(s.assets.text(ctx, Point2::new(2., 2.), &format!("Current level: {}", f.display()))?)
        } else {
            None
        };
        s.mplayer.play(ctx, Sound::Music)?;

        Ok(Box::new(Menu {
            title_txt: s.assets.text_big(ctx, Point2::new(w / 2., 16.), "Main Menu")?,
            play_btn: Button::new(ctx, &s.assets, button_rect(w, 0.), "Play")?,
            editor_btn: Button::new(ctx, &s.assets, button_rect(w, 1.), "Editor")?,
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
}

impl GameState for Menu {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.title_txt.draw_center(ctx)?;

        self.editor_btn.draw(ctx)?;
        if let Some(ref txt) = self.cur_lvl_text {
            txt.draw_text(ctx)?;
        }
        self.play_btn.draw(ctx)
    }
    fn key_up(&mut self, s: &mut State, ctx: &mut Context, keycode: Keycode) {
        use crate::Keycode::*;
        match keycode {
            P => self.switch_play(ctx, s),
            E => self.switch_editor(ctx, s),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, ctx: &mut Context, btn: MouseButton) {
        use crate::MouseButton::*;
        match btn {
            Left => {
                if self.play_btn.in_bounds(s.mouse) {
                    self.switch_play(ctx, s);
                }
                if self.editor_btn.in_bounds(s.mouse) {
                    self.switch_editor(ctx, s);
                }
            }
            _ => ()
        }
    }
}
