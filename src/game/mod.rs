use std::path::{Path, PathBuf};
use crate::{
    util::{Vector2, Point2},
    ext::BoolExt,
    io::{
        snd::MediaPlayer,
        tex::{Assets, PosText},
    },
    obj::{health::Health, weapon::WeaponInstance},
};
use ggez::{
    nalgebra::Matrix4,
    Context, GameResult,
    graphics::{self, DrawMode, Rect, Mesh, Text, DrawParam},
    timer,
    event::{EventHandler, MouseButton, KeyCode, KeyMods}
};
use clipboard::{ClipboardContext, ClipboardProvider};
use self::world::Level;

/// Stuff related to things in the world
pub mod world;
pub mod editor;
pub mod play;
pub mod menu;
pub mod lose;
pub mod win;

use self::menu::Menu;
use self::world::Statistics;

pub enum StateSwitch {
    Menu,
    Editor(Option<Level>),
    Play(Level),
    PlayWith{
        lvl: Box<Level>,
        health: Health,
        wep: Option<WeaponInstance<'static>>,
    },
    Lose(Box<Statistics>),
    Win(Box<Statistics>),
}

pub trait GameState {
    fn update(&mut self, _: &mut State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn logic(&mut self, _: &mut State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, _: &State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw_hud(&mut self, _: &State, _: &mut Context) -> GameResult<()>;
    fn key_down(&mut self, _: &mut State, _: &mut Context, _: KeyCode) {

    }
    fn key_up(&mut self, _: &mut State, _: &mut Context, _: KeyCode) {

    }
    fn mouse_down(&mut self, _: &mut State, _: &mut Context, _: MouseButton) {

    }
    fn mouse_up(&mut self, _: &mut State, _: &mut Context, _: MouseButton) {

    }

    fn get_world(&self) -> Option<&world::World> {
        None
    }
    fn get_mut_world(&mut self) -> Option<&mut world::World> {
        None
    }
}

const PROMPT_Y: f32 = 196.;

#[derive(Debug)]
pub struct Console {
    history: Text,
    prompt: PosText,
}

impl Console {
    fn new(_ctx: &mut Context, assets: &Assets) -> GameResult<Self> {
        Ok(Console {
            history: assets.raw_text_with("Welcome t' console\n", 18.),
            prompt: assets.text(Point2::new(0., PROMPT_Y)).and_text("> ").and_text(String::with_capacity(32)),
        })
    }
    fn execute(&mut self, ctx: &mut Context, state: &mut State, gs: &mut dyn GameState) -> GameResult<()> {
        let prompt = &mut self.prompt.text.fragments_mut()[1].text;

        self.history.add(format!("> {}\n", prompt));

        let cap = prompt.capacity();
        let prompt = mem::replace(prompt, String::with_capacity(cap));
        let args: Vec<_> = prompt.split(<char>::is_whitespace).collect();
        
        match args[0] {
            "" => (),
            "pi" => if let Some(world) = gs.get_mut_world() {
                world.intels.clear();
                self.history.add("Intels got\n");
            } else {
                self.history.add("No world\n");
            },
            "clear" => self.history = state.assets.raw_text_with("", 18.),
            "fa" => if let Some(world) = gs.get_mut_world() {
                world.player.health.hp = 100.;
                world.player.health.armour = 100.;
            } else {
                self.history.add("No world\n");
            },
            "god" => if let Some(world) = gs.get_mut_world() {
                if world.player.health.hp.is_finite() {
                    world.player.health.hp = std::f32::INFINITY;
                    self.history.add("Degreelessness\n");
                } else {
                    world.player.health.hp = 100.;
                    self.history.add("God off\n");
                }
            } else {
                self.history.add("No world\n");
            },
            "gg" => if let Some(world) = gs.get_mut_world() {
                world.player.utilities.grenades += 3;
                self.history.add("Gg'd\n");
            } else {
                self.history.add("No world\n");
            },
            "hello" => {
                self.history.add("Hello!\n");
            },
            "quit" => {
                ctx.continuing = false;
            }
            cmd => {
                self.history.add(format!("  Unknown command `{}'!\n", cmd));
            }
        }

        while self.history.height(ctx) > PROMPT_Y as u32 {
            let new_history = self.history.fragments().iter().skip(1).cloned().fold(state.assets.raw_text(18.), |mut text, f| {
                text.add(f);
                text
            });
            self.history = new_history;
        }

        Ok(())
    }
}

pub struct Master {
    gs: Box<dyn GameState>,
    state: State,
    console_open: bool,
    console: Console,
}

pub enum Content {
    Campaign(Campaign),
    File(PathBuf),
    None
}

/// The state of the game
pub struct State {
    assets: Assets,
    mplayer: MediaPlayer,
    width: f32,
    height: f32,
    mouse: Point2,
    offset: Vector2,
    switch_state: Option<StateSwitch>,
    content: Content,
}

const DESIRED_FPS: u32 = 60;

pub(crate) const DELTA: f32 = 1. / DESIRED_FPS as f32;

impl Master {
    #[allow(clippy::new_ret_no_self)]
    /// Make a new state object
    pub fn new(ctx: &mut Context, arg: &str) -> GameResult<Self> {
        // Initialise assets
        let assets = Assets::new(ctx)?;
        let mplayer = MediaPlayer::new(ctx)?;

        // Get the window's dimensions
        let Rect {w: width, h: height, ..} = graphics::screen_coordinates(ctx);

        let content;

        if arg.is_empty() {
            content = Content::None
        } else {
            content = Content::File(arg.to_owned().into())
        }

        let mut state = State {
            content,
            switch_state: None,
            assets,
            mplayer,
            width,
            height,
            mouse: Point2::new(0., 0.),
            offset: Vector2::new(0., 0.),
        };

        Ok(Master {
            console: Console::new(ctx, &state.assets)?,
            console_open: false,
            gs: Menu::new(ctx, &mut state)?,
            state,
        })
    }
}

impl State {
    /// Sets the offset so that the given point will be centered on the screen
    fn focus_on(&mut self, p: Point2) {
        self.offset = -p.coords + 0.5 * Vector2::new(self.width, self.height);
    }
    fn switch(&mut self, ss: StateSwitch) {
        self.switch_state = Some(ss);
    }
}

use std::mem;

impl EventHandler for Master {
    // Handle the game logic
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.console_open {
            while timer::check_update_time(ctx, DESIRED_FPS) {}

            Ok(())
        } else {
            if let Some(gsb) = mem::replace(&mut self.state.switch_state, None) {
                use self::StateSwitch::*;
                self.gs = match gsb {
                    PlayWith{lvl, health, wep} => play::Play::new(ctx, &mut self.state, *lvl, Some((health, wep))),
                    Play(lvl) => play::Play::new(ctx, &mut self.state, lvl, None),
                    Menu => menu::Menu::new(ctx, &mut self.state),
                    Editor(l) => editor::Editor::new(&self.state, l),
                    Win(stats) => win::Win::new(ctx, &mut self.state, *stats),
                    Lose(stats) => lose::Lose::new(ctx, &mut self.state, *stats),
                }?;
            }

            // Run this for every 1/60 of a second has passed since last update
            // Can in theory become slow
            while timer::check_update_time(ctx, DESIRED_FPS) {
                self.gs.update(&mut self.state, ctx)?;
            }
            self.gs.logic(&mut self.state, ctx)
        }
    }

    // Draws everything
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Clear the screen first
        graphics::clear(ctx, (33, 33, 255, 255).into());

        // Offset the current drawing with a translation from the `offset`
        graphics::push_transform(ctx, Some(Matrix4::new_translation(&self.state.offset.fixed_resize(0.))));
        graphics::apply_transformations(ctx)?;

        self.gs.draw(&self.state, ctx)?;

        // Pop the offset tranformation to draw the UI on the screen
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;

        self.gs.draw_hud(&self.state, ctx)?;

        if self.console_open {
            let console_bg = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(0., 0., self.state.width as f32, self.state.height as f32 / 3.), graphics::BLACK)?;
            graphics::draw(ctx, &console_bg, DrawParam::new())?;


            graphics::draw(ctx, &self.console.history, DrawParam::default())?;
            self.console.prompt.draw_text(ctx)?;
        }

        // Flip the buffers to see what we just drew
        graphics::present(ctx)?;

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    /// Handle key down events
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, km: KeyMods, repeat: bool) {
        // If this is a repeat event, we don't care
        if repeat {
            return
        }

        use self::KeyCode::*;
        // Update input axes and quit game on Escape
        match keycode {
            Escape if km.contains(KeyMods::SHIFT) => ctx.continuing = false,
            _ => (),
        }
        self.gs.key_down(&mut self.state, ctx, keycode)
    }
    /// Handle key release events
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods) {
        use self::KeyCode::*;
        if let Tab = keycode {
            self.console_open.toggle();
        }
        self.gs.key_up(&mut self.state, ctx, keycode)
    }
    fn text_input_event(&mut self, ctx: &mut Context, c: char) {
        if self.console_open {
            if c.is_control() {
                match c {
                    // Backspace
                    '\u{8}' => {self.console.prompt.text.fragments_mut()[1].text.pop();},
                    // Delete
                    '\u{7f}' => (),
                    // Escape
                    '\u{1b}' => (),
                    '\t' => (),
                    // Paste
                    '\u{16}' => {
                        let mut cc = ClipboardContext::new().unwrap();
                        let to_paste: String = cc.get_contents().unwrap();

                        self.console.prompt.text.fragments_mut()[1].text.push_str(&to_paste);
                    },
                    // Return (note sure whether newline is used on other platforms, so handling it in key_up)
                    '\r' => self.console.execute(ctx, &mut self.state, &mut *self.gs).unwrap(),
                    c => {self.console.history.add(format!("Unknown control character {:?}\n", c));}
                }
            } else {
                self.console.prompt.text.fragments_mut()[1].text.push(c);
            }
        }
    }
    /// Handle mouse down event
    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) {
        self.gs.mouse_down(&mut self.state, ctx, btn)
    }
    /// Handle mouse release events
    fn mouse_button_up_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) {
        self.gs.mouse_up(&mut self.state, ctx, btn)
    }
    /// Handles mouse movement events
    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32, _: f32, _: f32) {
        self.state.mouse = Point2::new(x, y);
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        false
    }
}


use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Campaign {
    pub levels: Vec<Level>,
    pub current: usize,
}

impl Campaign {
    pub fn load<P: AsRef<Path>>(p: P) -> GameResult<Self> {
        let file = BufReader::new(File::open(p)?);

        let mut levels = Vec::new();

        for line in file.lines() {
            let line = line?;
            levels.push(Level::load(&line.trim())?);
        }

        Ok(Campaign {
            levels,
            current: 0,
        })
    }
    pub fn next_level(&mut self) -> Option<Level> {
        let ret = self.levels.get(self.current).cloned();
        self.current += 1;
        ret
    }
}
