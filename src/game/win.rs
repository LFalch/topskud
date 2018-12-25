use crate::*;
use crate::io::btn::Button;
use crate::graphics::Rect;
use crate::game::world::Statistics;

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
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<GameState>> {
        let w = s.width as f32;

        let level_complete = s.assets.text(ctx, Point2::new(s.width as f32/ 2., 10.), "LEVEL COMPLETE")?;
        let hits_text = s.assets.text(ctx, Point2::new(4., 20.), &format!("Hits: {}", stats.hits))?;
        let misses_text = s.assets.text(ctx, Point2::new(4., 36.), &format!("Misses: {}", stats.misses))?;
        let enemies_text = s.assets.text(ctx, Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left))?;
        let health_text = s.assets.text(ctx, Point2::new(4., 68.), &format!("Health left: {}", stats.health_left))?;
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
        use crate::Keycode::*;
        match keycode {
            Return => s.switch(StateSwitch::Play),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use crate::MouseButton::*;
        match btn {
            Left => if self.continue_btn.in_bounds(s.mouse) {
                s.switch(StateSwitch::Play)
            }
            _ => ()
        }
    }
}
