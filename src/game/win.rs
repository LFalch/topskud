use crate::{
    util::Point2,
    io::{
        tex::PosText,
        btn::Button,
    },
};
use ggez::{
    Context, GameResult,
    graphics::{self, Rect},
    event::{MouseButton, Keycode}
};

use super::{State, GameState, StateSwitch, world::Statistics};

/// The state of the game
pub struct Win {
    level_complete: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    health_text: PosText,
    continue_btn: Button,
}

impl Win {
    #[allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<GameState>> {
        let w = s.width as f32;

        let level_complete = s.assets.text(ctx, Point2::new(s.width as f32/ 2., 10.), "LEVEL COMPLETE")?;
        let hits_text = s.assets.text(ctx, Point2::new(4., 20.), &format!("Hits: {}", stats.hits))?;
        let misses_text = s.assets.text(ctx, Point2::new(4., 36.), &format!("Misses: {}", stats.misses))?;
        let enemies_text = s.assets.text(ctx, Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left))?;
        let health_text = s.assets.text(ctx, Point2::new(4., 68.), &format!("Health left: {:02.0} / {:02.0}", stats.health_left.hp, stats.health_left.armour))?;
        let continue_btn = Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Continue")?;

        Ok(Box::new(Win {
            level_complete,
            hits_text,
            misses_text,
            enemies_text,
            health_text,
            continue_btn,
        }))
    }
}

impl GameState for Win {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.continue_btn.draw(ctx)?;

        self.level_complete.draw_center(ctx)?;
        graphics::set_color(ctx, graphics::BLACK)?;
        self.hits_text.draw_text(ctx)?;
        self.misses_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)?;
        self.health_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use self::Keycode::*;
        if let Return = keycode { s.switch(StateSwitch::Play) }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use self::MouseButton::*;
        if let Left = btn {
            if self.continue_btn.in_bounds(s.mouse) {
                s.switch(StateSwitch::Play)
            }
        }
    }
}
