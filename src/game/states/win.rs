use crate::{
    util::Point2,
    io::{
        tex::PosText,
        btn::Button,
    },
    obj::{health::Health, weapon::WeaponInstance},
    game::{
        DELTA,
        State, Content, GameState, StateSwitch, world::{Level, Statistics},
        event::{Event::{self, Key, Mouse}, MouseButton, KeyCode},
    }
};
use ggez::{
    Context, GameResult,
    graphics::Rect,
};

#[allow(clippy::large_enum_variant)]
enum WinButtons {
    CampaignMode {
        continue_btn: Button<()>
    },
    FileMode {
        restart_btn: Button<()>,
        edit_btn: Button<()>,
    }
}

/// The state of the game
pub struct Win {
    level_complete: PosText,
    time_text: PosText,
    enemies_text: PosText,
    health_text: PosText,
    buttons: WinButtons,
    health: Health,
    level: Level,
    weapon: Option<WeaponInstance<'static>>
}

impl Win {
    #[allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]
    pub fn new(ctx: &mut Context, s: &mut State, stats: Statistics) -> GameResult<Box<dyn GameState>> {
        let w = s.width as f32;

        let level_complete = s.assets.text(Point2::new(s.width as f32/ 2., 10.)).and_text("LEVEL COMPLETE");
        let time_text = s.assets.text(Point2::new(4., 20.)).and_text(format!("Time: {:.1}s", stats.time as f32 * DELTA));
        let enemy_total = stats.level.enemies.len();
        let enemies_text = s.assets.text(Point2::new(4., 36.)).and_text(format!("Enemies killed: {} / {}", enemy_total - stats.enemies_left, enemy_total));
        let health_text = s.assets.text(Point2::new(4., 52.)).and_text(format!("Health left: {:02.0} / {:02.0}", stats.health_left.hp, stats.health_left.armour));

        Ok(Box::new(Win {
            buttons: {
                match s.content {
                    Content::File(_) => WinButtons::FileMode {
                        restart_btn: Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Restart", ())?,
                        edit_btn: Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 132., w: w / 7., h: 64.}, "Edit", ())?,
                    },
                    _ => WinButtons::CampaignMode {
                        continue_btn: Button::new(ctx, &s.assets, Rect{x: 3. * w / 7., y: 64., w: w / 7., h: 64.}, "Continue", ())?,
                    }
                }
            },
            level_complete,
            time_text,
            enemies_text,
            health_text,
            level: stats.level,
            health: stats.health_left,
            weapon: stats.weapon,
        }))
    }
    fn restart(&self, s: &mut State) {
        s.switch(StateSwitch::Play(self.level.clone()));
    }
    fn edit(&self, s: &mut State) {
        s.switch(StateSwitch::Editor(Some(self.level.clone())));
    }
    fn continue_play(&self, s: &mut State) {
        let lvl;
        match &mut s.content {
            Content::Campaign(cam) => {
                if let Some(l) = cam.next_level() {
                    lvl = l;
                } else {
                    return
                }
            }
            Content::None | Content::File(_) => return,
        }

        s.switch(StateSwitch::PlayWith{health: self.health, wep: self.weapon, lvl: Box::new(lvl)});
    }
}

impl GameState for Win {
    fn draw_hud(&mut self, _s: &State, ctx: &mut Context) -> GameResult<()> {
        match &self.buttons {
            WinButtons::FileMode{restart_btn, edit_btn} => {
                restart_btn.draw(ctx)?;
                edit_btn.draw(ctx)?;
            }
            WinButtons::CampaignMode{continue_btn} => {
                continue_btn.draw(ctx)?;
            }
        }

        self.level_complete.draw_center(ctx)?;
        self.time_text.draw_text(ctx)?;
        self.enemies_text.draw_text(ctx)?;
        self.health_text.draw_text(ctx)
    }
    fn event_up(&mut self, s: &mut State, _ctx: &mut Context, event: Event) {
        use self::KeyCode::*;
        match event {
            Key(Return) => self.continue_play(s),
            Mouse(MouseButton::Left) => match &self.buttons {
                WinButtons::FileMode{restart_btn, edit_btn} => {
                    if restart_btn.in_bounds(s.mouse) {
                        self.restart(s)
                    }
                    if edit_btn.in_bounds(s.mouse) {
                        self.edit(s)
                    }
                }
                WinButtons::CampaignMode{continue_btn} => {
                    if continue_btn.in_bounds(s.mouse) {
                        self.continue_play(s)
                    }
                }
            }
            _ => (),
        }
    }
}
