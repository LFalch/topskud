use topskud::{
    DELTA,
    util::RED,
    io::{
        tex::PosText,
        btn::Button,
        ctrl::Input,
    },
    world::{Statistics, Level},
    obj::{health::Health, player::WepSlots},
};
use crate::game::{
    State, Content, GameState, StateSwitch,
};
use ggez::{
    Context, GameResult,
    graphics::{Rect, TextFragment, Canvas},
};

/// The state of the game
pub struct Lose {
    you_died: PosText,
    time_text: PosText,
    enemies_text: PosText,
    restart_btn: Button<()>,
    edit_btn: Option<Button<()>>,
    level: Level,
    health: Health,
    weapon: WepSlots
}

impl Lose {
    #[allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<dyn GameState>> {
        let w = s.width as f32;
        let you_died = s.assets.text(point!(s.width as f32/ 2., 10.)).and_text(TextFragment::from("You died!").color(RED)).centered();
        let time_text = s.assets.text(point!(4., 20.)).and_text(format!("Time: {:.0}s", stats.time as f32 * DELTA));
        let enemy_total = stats.level.enemies.len();
        let enemies_text = s.assets.text(point!(4., 36.)).and_text(format!("Enemies killed: {} / {}", enemy_total - stats.enemies_left, enemy_total));
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
            time_text,
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
        s.switch(StateSwitch::PlayWith{lvl: Box::new(self.level.clone()), health: self.health, wep: self.weapon.clone()})
    }
}

impl GameState for Lose {
    fn draw_hud(&mut self, _s: &State, canvas: &mut Canvas, _ctx: &mut Context) -> GameResult<()> {
        self.restart_btn.draw(canvas);
        if let Some(btn) = &self.edit_btn {
            btn.draw(canvas);
        }

        self.you_died.draw_text(canvas);
        self.time_text.draw_text(canvas);
        self.enemies_text.draw_text(canvas);

        Ok(())
    }
    fn event_up(&mut self, s: &mut State, _ctx: &mut Context, input: Input) {
        match input {
            Input::Confirm | Input::Restart => self.restart(s),
            Input::LeftClick => {
                if self.restart_btn.in_bounds(s.mouse) {
                    self.restart(s);
                }
                if let Some(btn) = &self.edit_btn {
                    if btn.in_bounds(s.mouse) {
                        self.edit(s);
                    }
                }
            } 
            _ => (),
        }

    }
}
