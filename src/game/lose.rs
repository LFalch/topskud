use crate::*;
use crate::io::btn::Button;
use crate::graphics::Rect;
use crate::game::world::Statistics;

/// The state of the game
pub struct Lose {
    you_died: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    restart_btn: Button,
}

impl Lose {
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
        use crate::Keycode::*;
        match keycode {
            R | Return => s.switch(StateSwitch::Play),
            _ => (),
        }
    }
    fn mouse_up(&mut self, s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        use crate::MouseButton::*;
        match btn {
            Left => if self.restart_btn.in_bounds(s.mouse) {
                    s.switch(StateSwitch::Play);
                }
            _ => ()
        }
    }
}
