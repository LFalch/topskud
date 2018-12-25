use crate::{
    util::{RED, Point2},
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
pub struct Lose {
    you_died: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    restart_btn: Button,
}

impl Lose {
    #[allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<GameState>> {
        let w = s.width as f32;
        let you_died = s.assets.text(ctx, Point2::new(s.width as f32/ 2., 10.), "You died!")?;
        let hits_text = s.assets.text(ctx, Point2::new(4., 20.), &format!("Hits: {}", stats.hits))?;
        let misses_text = s.assets.text(ctx, Point2::new(4., 36.), &format!("Misses: {}", stats.misses))?;
        let enemies_text = s.assets.text(ctx, Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left))?;
        let restart_btn = Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Restart")?;

        Ok(Box::new(Lose {
            you_died,
            hits_text,
            misses_text,
            enemies_text,
            restart_btn,
        }))
    }
}

impl GameState for Lose {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.restart_btn.draw(ctx)?;

        graphics::set_color(ctx, RED)?;
        self.you_died.draw_center(ctx)?;
        graphics::set_color(ctx, graphics::BLACK)?;
        self.hits_text.draw_text(ctx)?;
        self.misses_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use self::Keycode::*;
        match keycode {
            R | Return => s.switch(StateSwitch::Play),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use self::MouseButton::*;
        if let Left = btn {
            if self.restart_btn.in_bounds(s.mouse) {
                s.switch(StateSwitch::Play);
            }
        }
    }
}
