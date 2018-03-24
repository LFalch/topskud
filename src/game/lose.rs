use ::*;
use graphics::{Rect, DrawMode};
use game::world::Statistics;

/// The state of the game
pub struct Lose {
    you_died: PosText,
    hits_text: PosText,
    misses_text: PosText,
    enemies_text: PosText,
    restart_text: PosText,
}

impl Lose {
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<GameState>> {
        let you_died = s.assets.text(ctx, Point2::new(s.width as f32/ 2., 10.), "You died!")?;
        let hits_text = s.assets.text(ctx, Point2::new(4., 20.), &format!("Hits: {}", stats.hits))?;
        let misses_text = s.assets.text(ctx, Point2::new(4., 36.), &format!("Misses: {}", stats.misses))?;
        let enemies_text = s.assets.text(ctx, Point2::new(4., 52.), &format!("Enemies left: {}", stats.enemies_left))?;
        let restart_text = s.assets.text(ctx, Point2::new(s.width as f32/2., 96.), "Restart")?;

        Ok(Box::new(Lose {
            you_died,
            hits_text,
            misses_text,
            enemies_text,
            restart_text,
        }))
    }
    pub fn rect(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) -> GameResult<()> {
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x, y, h, w})
    }
}

impl GameState for Lose {
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        let w = s.width as f32;
        let x = 3. * w / 7.;
        let w = w / 7.;
        graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.75, a: 1.})?;
        Lose::rect(ctx, x, 64., w, 64.)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        self.restart_text.draw_center(ctx)?;

        graphics::set_color(ctx, RED)?;
        self.you_died.draw_center(ctx)?;
        graphics::set_color(ctx, graphics::BLACK)?;
        self.hits_text.draw_text(ctx)?;
        self.misses_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, _ctx: &mut Context, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            R | Return => s.switch(StateSwitch::Play),
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
