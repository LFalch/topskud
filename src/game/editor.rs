use ::*;
use graphics::{Rect, DrawMode};
use super::world::*;
use super::play::Play;

/// The state of the game
pub struct Editor {
    pos: Point2,
    level: Level,
    cur_mat: Material,
    current_mat_text: PosText,
}

impl Editor {
    pub fn new(assets: &Assets, ctx: &mut Context) -> GameResult<Self> {
        // Initialise the text objects
        let current_mat_text = assets.text(ctx, Point2::new(2., 18.0), "Current material: {:?}")?;

        Ok(Editor {
            pos: Point2::new(0., 0.),
            cur_mat: Material::Wall,
            current_mat_text,
            level: Level::new(),
        })
    }
    /// Update the text objects
    fn update_ui(&mut self, s: &State, ctx: &mut Context) {
        let current_mat_str = format!("Current material:");

        self.current_mat_text.update_text(&s.assets, ctx, &current_mat_str).unwrap();
    }
}


impl GameState for Editor {
    fn update(&mut self, s: &mut State) {
        let v = 175. * Vector2::new(s.input.hor(), s.input.ver());
        self.pos += v * DELTA;
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) {
        if s.mouse_down.left {
            let (mx, my) = Level::snap(s.mouse - s.offset);
            self.level.insert(mx, my, self.cur_mat);
        }

        // Update the UI
        self.update_ui(&s, ctx);
        s.focus_on(self.pos);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.level.draw(ctx, &s.assets)?;

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x:0.,y:0.,h: 64., w: s.width as f32})?;
        graphics::set_color(ctx, graphics::WHITE)?;
        self.cur_mat.draw(ctx, &s.assets, 186., 16.)?;

        self.current_mat_text.draw_text(ctx)
    }
    fn key_up(&mut self, s: &mut State, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            Space => self.cur_mat = match self.cur_mat {
                Material::Wall => Material::Floor,
                Material::Floor => Material::Grass,
                Material::Grass => Material::Wall,
            },
            Z => save::save("save.lvl", &self.level).unwrap(),
            X => save::load("save.lvl", &mut self.level).unwrap(),
            P => s.switch(Box::new(Play::new(self.level.clone()))),
            _ => return,
        }
    }
}
