use std::path::PathBuf;
use crate::{
    util::Point2,
    io::{
        tex::PosText,
        btn::Button,
        snd::Sound,
    },
};
use ggez::{
    Context, GameResult,
    graphics::{self, Rect},
    event::{MouseButton}
};

use super::{Campaign, Content, State, GameState, StateSwitch, world::Level};

/// The state of the game
pub struct Menu {
    title_txt: PosText,
    buttons: Vec<Button<Callback>>,
    corner_text: Option<PosText>,
}

enum Callback {
    SwitchPlay(PathBuf),
    SwitchEditor,
    Campaign(PathBuf),
}

// ↓
fn button_rect(w: f32, i: f32) -> Rect {
    Rect{x:3. * w / 7., y: 64. + i * 68., w:w / 7., h:64.}
}

impl Menu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State) -> GameResult<Box<dyn GameState>> {
        let w = s.width as f32;

        let corner_text = if let Content::File(ref f) = s.content {
            Some(s.assets.text(ctx, Point2::new(2., 2.), &format!("File: {}", f.display()))?)
        } else {
            None
        };
        s.mplayer.play(ctx, Sound::Music)?;

        let buttons = match &mut s.content {
            Content::Campaign(_cam) => {
                unreachable!()
            }
            Content::File(p) if p.extension().and_then(|s| s.to_str()) == Some("cmp") => {
                vec![
                    Button::new(ctx, &s.assets, button_rect(w, 0.), "Play campaign", Callback::Campaign(p.clone()))?,
                ]
            }
            Content::File(p) => {
                vec![
                    Button::new(ctx, &s.assets, button_rect(w, 0.), "Play", Callback::SwitchPlay(p.clone()))?,
                    Button::new(ctx, &s.assets, button_rect(w, 1.), "Editor", Callback::SwitchEditor)?,
                ]
            }
            Content::None => {
                std::fs::read_dir("campaigns/")?
                    .filter_map(Result::ok)
                    .enumerate()
                    .map(|(i, d)| Button::new(
                        ctx, &s.assets, button_rect(w, i as f32), d.file_name().to_str().unwrap(), Callback::Campaign(d.path())
                    ))
                    .filter_map(Result::ok)
                    .collect()
            },
        };

        Ok(Box::new(Menu {
            title_txt: s.assets.text_big(ctx, Point2::new(w / 2., 16.), "Main Menu")?,
            buttons,
            corner_text,
        }))
    }
}

impl GameState for Menu {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.title_txt.draw_center(ctx)?;
        if let Some(ref txt) = self.corner_text {
            txt.draw_text(ctx)?;
        }
        for button in &self.buttons {
            button.draw(ctx)?;
        }
        Ok(())
    }
    // fn key_up(&mut self, s: &mut State, ctx: &mut Context, keycode: Keycode) {
    //     use self::Keycode::*;
    //     match keycode {
    //         P => self.switch_play(ctx, s),
    //         E => self.switch_editor(ctx, s),
    //         _ => (),
    //     }
    // }
    fn mouse_up(&mut self, s: &mut State, ctx: &mut Context, btn: MouseButton) {
        use self::MouseButton::*;
        if let Left = btn {
            for button in &self.buttons {
                if button.in_bounds(s.mouse) {
                    s.mplayer.stop(ctx, Sound::Music).unwrap();
                    match &button.callback {
                        Callback::Campaign(cam) => {
                            let mut cam = Campaign::load(cam).unwrap();
                            let lvl = cam.next_level().unwrap();
                            s.content = Content::Campaign(cam);
                            s.switch(StateSwitch::Play(lvl));
                        },
                        Callback::SwitchPlay(p) => {
                            let lvl = Level::load(&p).unwrap();
                            s.switch(StateSwitch::Play(lvl));
                        },
                        Callback::SwitchEditor => s.switch(StateSwitch::Editor(None)),
                    }
                }
            }
        }
    }
}
