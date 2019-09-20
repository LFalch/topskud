use crate::{
    util::{RED, Point2},
    io::{
        tex::PosText,
        btn::Button,
    },
    obj::{health::Health, weapon::WeaponInstance},
};
use ggez::{
    Context, GameResult,
    graphics::{self, Rect},
    event::{MouseButton, KeyCode}
};

use super::{State, Content, GameState, StateSwitch, world::{Statistics, Level}};

/// The state of the game
pub struct Lose {
    you_died: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    restart_btn: Button<()>,
    edit_btn: Option<Button<()>>,
    level: Level,
    health: Health,
    weapon: Option<WeaponInstance<'static>>
}

impl Lose {
    #[allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<dyn GameState>> {
        let w = s.width as f32;
        let you_died = s.assets.text(Point2::new(s.width as f32/ 2., 10.), "You died!");
        let hits_text = s.assets.text(Point2::new(4., 20.), &format!("Hits: {}", stats.hits));
        let misses_text = s.assets.text(Point2::new(4., 36.), &format!("Misses: {}", stats.misses));
        let enemies_text = s.assets.text(Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left));
        let restart_btn = Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Restart", ())?;
        let edit_btn = if let Content::File(_) = s.content {
            Some(
                Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 132., w: w / 7., h: 64.}, "Edit", ())?
            )
        } else {
            None
        };

        Ok(Box::new(Lose {
            you_died,
            hits_text,
            misses_text,
            enemies_text,
            restart_btn,
            edit_btn,
            level: stats.level,
            health: stats.health_left,
            weapon: stats.weapon,
        }))
    }
    fn edit(&self, s: &mut State) {
        s.switch(StateSwitch::Editor(Some(self.level.clone())));
    }
    fn restart(&self, s: &mut State) {
        s.switch(StateSwitch::PlayWith{lvl: Box::new(self.level.clone()), health: self.health, wep: self.weapon})
    }
}

impl GameState for Lose {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        self.restart_btn.draw(ctx)?;
        if let Some(btn) = &self.edit_btn {
            btn.draw(ctx)?;
        }

        // graphics::set_color(ctx, RED)?;
        self.you_died.draw_center(ctx)?;
        // graphics::set_color(ctx, graphics::BLACK)?;
        self.hits_text.draw_text(ctx)?;
        self.misses_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: KeyCode) {
        use self::KeyCode::*;
        match keycode {
            R | Return => self.restart(s),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use self::MouseButton::*;
        if let Left = btn {
            if self.restart_btn.in_bounds(s.mouse) {
                self.restart(s);
            }
            if let Some(btn) = &self.edit_btn {
                if btn.in_bounds(s.mouse) {
                    self.edit(s);
                }
            }
        }
    }
}
