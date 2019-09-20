use std::path::{Path, PathBuf};
use crate::{
    util::{Vector2, Point2},
    ext::{MouseDown, InputState, Modifiers, BoolExt},
    io::{
        snd::MediaPlayer,
        tex::{Assets, PosText},
    },
    obj::{health::Health, weapon::WeaponInstance},
};
use ggez::{
    nalgebra::Matrix4,
    Context, GameResult,
    graphics::{self, DrawMode, Rect, Mesh, Text},
    timer,
    event::{EventHandler, MouseButton, KeyCode, KeyMods}
};
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
    prompt_str: String,
}

impl Console {
    fn new(ctx: &mut Context, assets: &Assets) -> GameResult<Self> {
        Ok(Console {
            history: Text::new("Welcome t' console".into()),
            prompt: assets.text(Point2::new(0., PROMPT_Y), "> "),
            prompt_str: String::with_capacity(32),
        })
    }
    fn execute(&mut self, ctx: &mut Context, state: &mut State, gs: &mut dyn GameState) -> GameResult<()> {
        self.history.add(format!("> {}", self.prompt_str));

        let cap = self.prompt_str.capacity();
        let prompt = mem::replace(&mut self.prompt_str, String::with_capacity(cap));
        let args: Vec<_> = prompt.split(' ').collect();
        
        match args[0] {
            "" => (),
            "pi" => if let Some(world) = gs.get_mut_world() {
                world.intels.clear();
                self.history.add("Intels got\n");
            } else {
                self.history.add("No world\n");
            },
            "clear" => self.history = Text::new("".into()),
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
            }
            cmd => {
                self.history.add(format!("  Unknown command `{}'!\n", cmd));
            }
        }

        while self.history.height(ctx) > PROMPT_Y as u32 {
            *self.history.fragments_mut().first_mut().unwrap() = "".into();
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
    mouse_down: MouseDown,
    input: InputState,
    modifiers: Modifiers,
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
            input: Default::default(),
            mouse_down: Default::default(),
            modifiers: Default::default(),
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
            if self.console.prompt_str != self.console.prompt.text.contents()[2..] {
                self.console.prompt.update_text(&self.state.assets, ctx, &format!("> {}", self.console.prompt_str))?;
            }

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
            graphics::draw(ctx, &console_bg, Default::default())?;


            graphics::draw(ctx, &self.console.history, Default::default())?;
            self.console.prompt.draw_text(ctx)?;
        }

        // Flip the buffers to see what we just drew
        graphics::present(ctx);

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    /// Handle key down events
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, repeat: bool) {
        // If this is a repeat event, we don't care
        if repeat {
            return
        }

        use self::KeyCode::*;
        // Update input axes and quit game on Escape
        match keycode {
            W | Up => self.state.input.ver -= 1,
            S | Down => self.state.input.ver += 1,
            A | Left => self.state.input.hor -= 1,
            D | Right => self.state.input.hor += 1,
            LShift => self.state.modifiers.shift = true,
            LCtrl => self.state.modifiers.ctrl = true,
            LAlt => self.state.modifiers.alt = true,
            Escape if self.console_open => self.console_open = false,
            Escape => ctx.continuing = false,
            _ => (),
        }
        self.gs.key_down(&mut self.state, ctx, keycode)
    }
    /// Handle key release events
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods) {
        use self::KeyCode::*;
        match keycode {
            W | Up => self.state.input.ver += 1,
            S | Down => self.state.input.ver -= 1,
            A | Left => self.state.input.hor += 1,
            D | Right => self.state.input.hor -= 1,
            LShift => self.state.modifiers.shift = false,
            LCtrl => self.state.modifiers.ctrl = false,
            LAlt => self.state.modifiers.alt = false,
            Less => self.console_open.toggle(),
            Backspace if self.console_open => {self.console.prompt_str.pop();},
            Return if self.console_open => self.console.execute(ctx, &mut self.state, &mut *self.gs).unwrap(),
            _ => (),
        }
        self.gs.key_up(&mut self.state, ctx, keycode)
    }
    fn text_input_event(&mut self, _ctx: &mut Context, c: char) {
        if self.console_open {
            self.console.prompt_str.push(c);
        }
    }
    /// Handle mouse down event
    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) {
        use self::MouseButton::*;
        match btn {
            Left => self.state.mouse_down.left = true,
            Middle => self.state.mouse_down.middle = true,
            Right => self.state.mouse_down.right = true,
            _ => ()
        }
        self.gs.mouse_down(&mut self.state, ctx, btn)
    }
    /// Handle mouse release events
    fn mouse_button_up_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) {
        use self::MouseButton::*;
        match btn {
            Left => self.state.mouse_down.left = false,
            Middle => self.state.mouse_down.middle = false,
            Right => self.state.mouse_down.right = false,
            _ => ()
        }
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
