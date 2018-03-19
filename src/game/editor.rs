use ::*;
use graphics::{Rect, DrawMode};
use super::world::*;
use super::play::Play;

/// The state of the game
pub struct Editor {
    pos: Point2,
    fast: bool,
    level: Level,
    cur_mat: Material,
    current_mat_text: PosText,
}

impl Editor {
    pub fn new(assets: &Assets, ctx: &mut Context) -> GameResult<Self> {
        // Initialise the text objects
        let current_mat_text = assets.text(ctx, Point2::new(2., 18.0), "Current material: {:?}")?;

        Ok(Editor {
            fast: false,
            pos: Point2::new(200., 200.),
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
        let speed = match self.fast {
            false => 175.,
            true => 315.,
        };
        let v = speed * Vector2::new(s.input.hor(), s.input.ver());
        self.pos += v * DELTA;
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) {
        if s.mouse_down.left {
            let (mx, my) = Grid::snap(s.mouse - s.offset);
            self.level.grid.insert(mx, my, self.cur_mat);
        }

        // Update the UI
        self.update_ui(&s, ctx);
        s.focus_on(self.pos);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::WHITE)?;
        self.level.grid.draw(ctx, &s.assets)?;
        if let Some(start) = self.level.start_point {
            graphics::set_color(ctx, GREEN)?;
            graphics::circle(ctx, DrawMode::Fill, start, 16., 1.)?;
            graphics::set_color(ctx, BLUE)?;
            graphics::circle(ctx, DrawMode::Fill, start, 9., 1.)?;
        }

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
                Material::Grass => Material::Dirt,
                Material::Dirt => Material::Wall,
                Material::Missing => Material::Wall,
            },
            Z => self.level.save("save.lvl").unwrap(),
            X => self.level = Level::load("save.lvl").unwrap(),
            P => s.switch(Box::new(Play::new(self.level.clone()))),
            LShift => self.fast = false,
            _ => return,
        }
    }
    fn mouse_up(&mut self, s: &mut State, btn: MouseButton) {
        if let MouseButton::Right = btn {
            self.level.start_point = Some(s.mouse - s.offset);
        }
    }
    fn key_down(&mut self, _s: &mut State, keycode: Keycode) {
        use Keycode::*;
        match keycode {
            LShift => self.fast = true,
            _ => return,
        }
    }
}
