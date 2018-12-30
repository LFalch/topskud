use crate::{
    util::Point2,
    io::{
        tex::PosText,
        btn::Button,
    },
    obj::{health::Health, weapon::WeaponInstance},
};
use ggez::{
    Context, GameResult,
    graphics::{self, Rect},
    event::{MouseButton, Keycode}
};

use super::{State, Content, GameState, StateSwitch, world::{Level, Statistics}};

enum WinButtons {
    CampaignMode {
        continue_btn: Button<()>
    },
    FileMode {
        restart_btn: Button<()>,
        edit_btn: Button<()>,
    }
}

/// The state of the game
pub struct Win {
    level_complete: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    health_text: PosText,
    buttons: WinButtons,
    health: Health,
    level: Level,
    weapon: Option<WeaponInstance<'static>>
}

impl Win {
    #[allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<dyn GameState>> {
        let w = s.width as f32;

        let level_complete = s.assets.text(ctx, Point2::new(s.width as f32/ 2., 10.), "LEVEL COMPLETE")?;
        let hits_text = s.assets.text(ctx, Point2::new(4., 20.), &format!("Hits: {}", stats.hits))?;
        let misses_text = s.assets.text(ctx, Point2::new(4., 36.), &format!("Misses: {}", stats.misses))?;
        let enemies_text = s.assets.text(ctx, Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left))?;
        let health_text = s.assets.text(ctx, Point2::new(4., 68.), &format!("Health left: {:02.0} / {:02.0}", stats.health_left.hp, stats.health_left.armour))?;

        Ok(Box::new(Win {
            buttons: {
                match s.content {
                    Content::File(_) => WinButtons::FileMode {
                        restart_btn: Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Restart", ())?,
                        edit_btn: Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 132., w: w / 7., h: 64.}, "Edit", ())?,
                    },
                    _ => WinButtons::CampaignMode {
                        continue_btn: Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Continue", ())?,
                    }
                }
            },
            level_complete,
            hits_text,
            misses_text,
            enemies_text,
            health_text,
            level: stats.level,
            health: stats.health_left,
            weapon: stats.weapon,
        }))
    }
    fn restart(&self, s: &mut State) {
        s.switch(StateSwitch::Play(self.level.clone()));
    }
    fn edit(&self, s: &mut State) {
        s.switch(StateSwitch::Editor(Some(self.level.clone())));
    }
    fn continue_play(&self, s: &mut State) {
        let lvl;
        match &mut s.content {
            Content::Campaign(cam) => {
                if let Some(l) = cam.next_level() {
                    lvl = l;
                } else {
                    return
                }
            }
            Content::None | Content::File(_) => return,
        }

        s.switch(StateSwitch::PlayWith{health: self.health, wep: self.weapon, lvl});
    }
}

impl GameState for Win {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        match &self.buttons {
            WinButtons::FileMode{restart_btn, edit_btn} => {
                restart_btn.draw(ctx)?;
                edit_btn.draw(ctx)?;
            }
            WinButtons::CampaignMode{continue_btn} => {
                continue_btn.draw(ctx)?;
            }
        }

        self.level_complete.draw_center(ctx)?;
        graphics::set_color(ctx, graphics::BLACK)?;
        self.hits_text.draw_text(ctx)?;
        self.misses_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)?;
        self.health_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use self::Keycode::*;
        if let Return = keycode { self.continue_play(s) }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use self::MouseButton::*;
        if let Left = btn {
            match &self.buttons {
                WinButtons::FileMode{restart_btn, edit_btn} => {
                    if restart_btn.in_bounds(s.mouse) {
                        self.restart(s)
                    }
                    if edit_btn.in_bounds(s.mouse) {
                        self.edit(s)
                    }
                }
                WinButtons::CampaignMode{continue_btn} => {
                    if continue_btn.in_bounds(s.mouse) {
                        self.continue_play(s)
                    }
                }
            }
        }
    }
}
