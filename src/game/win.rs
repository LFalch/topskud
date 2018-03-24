use ::*;
use graphics::{Rect, DrawMode};
use game::world::Statistics;

/// The state of the game
pub struct Win {
    level_complete: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    health_text: PosText,
    continue_text: PosText,
}

impl Win {
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<GameState>> {
        let level_complete = s.assets.text(ctx, Point2::new(s.width as f32/ 2., 10.), "LEVEL COMPLETE")?;
        let hits_text = s.assets.text(ctx, Point2::new(4., 20.), &format!("Hits: {}", stats.hits))?;
        let misses_text = s.assets.text(ctx, Point2::new(4., 36.), &format!("Misses: {}", stats.misses))?;
        let enemies_text = s.assets.text(ctx, Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left))?;
        let health_text = s.assets.text(ctx, Point2::new(4., 68.), &format!("Health left: {}", stats.health_left))?;
        let continue_text = s.assets.text(ctx, Point2::new(s.width as f32/2., 96.), "Continue")?;

        Ok(Box::new(Win {
            level_complete,
            hits_text,
            misses_text,
            enemies_text,
            health_text,
            continue_text,
        }))
    }
    pub fn rect(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) -> GameResult<()> {
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x, y, h, w})
    }
}

impl GameState for Win {
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        let w = s.width as f32;
        let x = 3. * w / 7.;
        let w = w / 7.;
        graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.75, a: 1.})?;
        Win::rect(ctx, x, 64., w, 64.)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        self.continue_text.draw_center(ctx)?;

        self.level_complete.draw_center(ctx)?;
        graphics::set_color(ctx, graphics::BLACK)?;
        self.hits_text.draw_text(ctx)?;
        self.misses_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)?;
        self.health_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            Return => s.switch(StateSwitch::Play),
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
                    if s.mouse.y >= 64. && s.mouse.y < 128. {
                        s.switch(StateSwitch::Play)
                    }
                }
            }
            _ => ()
        }
    }
}
