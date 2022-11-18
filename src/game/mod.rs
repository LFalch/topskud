use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::fmt::{self, Display};
use std::collections::HashMap;
use crate::{
    util::{Vector2, Point2, RED, GREEN, BLUE},
    io::{
        snd::MediaPlayer,
        tex::{Assets, PosText},
    },
    obj::{health::Health, player::WepSlots},
};
use ggez::graphics::{Canvas, Drawable};
use ggez::input::keyboard::{KeyInput, KeyMods};
use ggez::winit::event::VirtualKeyCode;
use ggez::{
    Context, GameResult,
    graphics::{self, DrawMode, Rect, Mesh, Text, TextFragment, DrawParam, Color},
    timer,
    input::mouse::{self, CursorIcon},
    event::EventHandler
};
use clipboard::{ClipboardContext, ClipboardProvider};
use self::world::Level;
use log::{Log, Metadata, Record, Level as LogLevel};
use lazy_static::lazy_static;

/// Stuff related to things in the world
pub mod world;
pub mod states;

use self::states::menu::Menu;
use self::world::Statistics;

pub enum StateSwitch {
    Menu,
    Editor(Option<Level>),
    Play(Level),
    PlayWith{
        lvl: Box<Level>,
        health: Health,
        wep: WepSlots,
    },
    Lose(Box<Statistics>),
    Win(Box<Statistics>),
}

pub mod event {
    pub use ggez::input::{
        mouse::MouseButton,
        keyboard::KeyCode,
    };
    pub enum Event {
        Key(KeyCode),
        Mouse(MouseButton),
    }
}

use event::*;

pub trait GameState {
    fn update(&mut self, _: &mut State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn logic(&mut self, _: &mut State, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, _: &State, _: &mut Canvas, _: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw_hud(&mut self, _: &State, _: &mut Canvas, _: &mut Context) -> GameResult<()>;
    fn event_down(&mut self, _: &mut State, _: &mut Context, _: Event) { }
    fn event_up(&mut self, _: &mut State, _: &mut Context, _: Event) { }

    fn get_world(&self) -> Option<&world::World> {
        None
    }
    fn get_mut_world(&mut self) -> Option<&mut world::World> {
        None
    }
}

lazy_static! {
    static ref CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger::default();
}

#[derive(Debug, Default)]
struct ConsoleLogger {
    fragments: Mutex<Vec<TextFragment>>
}

impl ConsoleLogger {
    #[inline]
    fn get_colour(l: LogLevel) -> Option<Color> {
        use self::LogLevel::*;
        match l {
            Trace => Some(BLUE),
            Info => None,
            Debug => Some(GREEN),
            Warn => Some(Color{r:1.,g:1.,b:0.,a:1.}),
            Error => Some(RED),
        }
    }
    fn empty(&self) -> impl Iterator<Item=TextFragment> {
        let frags = std::mem::replace(&mut *self.fragments.lock().unwrap(), Vec::new());
        
        frags.into_iter()
    }
}

impl Log for ConsoleLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Only want to deal with logs from this crate
        metadata.target().starts_with("topskud")
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            print!("{}: ", record.level());

            println!("{}", record.args());

            let frag: TextFragment = format!("{}\n", record.args()).into();

            let mut frags = self.fragments.lock().unwrap();

            if let Some(color) = Self::get_colour(record.level()) {
                frags.push(frag.color(color));
            } else {
                frags.push(frag);
            }
        }
    }
    fn flush(&self) {}
}

const PROMPT_Y: f32 = 196.;

pub struct Console {
    commands: HashMap<String, Command>,
    history: Text,
    prompt: PosText,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandError {
    NoWorld,
    NoCampaign,
    InvalidArg,
    NoSuchLevel,
    NoSuchWeapon,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CommandError::*;
        match *self {
            NoWorld => "No world".fmt(f),
            NoCampaign => "No campaign loaded".fmt(f),
            InvalidArg => "Invalid argument".fmt(f),
            NoSuchLevel => "No such level".fmt(f),
            NoSuchWeapon => "No such weapon".fmt(f),
        }
    }
}

mod cmds;

type Command = for<'a> fn(console: &'a mut Console, ctx: &'a mut Context, state: &'a mut State, gs: &'a mut dyn GameState, args: Vec<&'a str>) -> Result<(), CommandError>;

impl Console {
    fn new(_ctx: &mut Context, assets: &Assets) -> GameResult<Self> {
        log::set_logger(&*CONSOLE_LOGGER).expect("to be first logger");
        log::set_max_level(log::LevelFilter::Trace);

        Ok(Console {
            commands: cmds::commands(),
            history: assets.raw_text_with("Acheivements disabled.\n", 18.),
            prompt: assets.text(point!(0., PROMPT_Y)).and_text("> ").and_text(String::with_capacity(32)),
        })
    }
    fn execute(&mut self, ctx: &mut Context, state: &mut State, gs: &mut dyn GameState) -> GameResult<()> {
        let prompt = &mut self.prompt.text.fragments_mut()[1].text;

        self.history.add(format!("> {}\n", prompt));

        let cap = prompt.capacity();
        let prompt = mem::replace(prompt, String::with_capacity(cap));
        let args: Vec<_> = prompt.split(<char>::is_whitespace).collect();

        let command_name = args[0];

        match self.commands.get(command_name) {
            Some(cmd) => if let Err(e) = cmd(self, ctx, state, gs, args) {
                error!("{}", e);
            },
            None => warn!("  Unknown command `{}'!", command_name),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleStatus {
    Open {
        cursor: CursorIcon,
        cursor_hidden: bool,
    },
    Closed
}

impl ConsoleStatus {
    pub fn is_open(self) -> bool {
        if let ConsoleStatus::Open{..} = self {
            true
        } else {
            false
        }
    }
    pub fn open(&mut self, ctx: &Context) {
        if let ConsoleStatus::Closed = self {
            *self = ConsoleStatus::Open{
                cursor: ctx.mouse.cursor_type(),
                cursor_hidden: ctx.mouse.cursor_hidden(),
            };
        }
    }
    pub fn close(&mut self, ctx: &mut Context) {
        if let ConsoleStatus::Open{cursor, cursor_hidden} = std::mem::replace(self, ConsoleStatus::Closed) {
            mouse::set_cursor_type(ctx, cursor);
            mouse::set_cursor_hidden(ctx, cursor_hidden);
        }
    }
}

pub struct Master {
    gs: Box<dyn GameState>,
    state: State,
    console_status: ConsoleStatus,
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
    /// Make a new state object
    pub fn new(ctx: &mut Context, arg: &str) -> GameResult<Self> {
        // Initialise assets
        let assets = Assets::new(ctx)?;
        let mut mplayer = MediaPlayer::new();
        mplayer.register_music(ctx, "music", true)?;
        mplayer.register_music(ctx, "victory", false)?;


        let canvas = Canvas::from_frame(ctx, None);
        // Get the window's dimensions
        let Rect {w: width, h: height, ..} = canvas.screen_coordinates().unwrap();
        drop(canvas);

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
            mouse: point![0., 0.],
            offset: vector![0., 0.],
        };

        Ok(Master {
            console: Console::new(ctx, &state.assets)?,
            console_status: ConsoleStatus::Closed,
            gs: Menu::new(ctx, &mut state)?,
            state,
        })
    }
}

impl State {
    /// Sets the offset so that the given point will be centered on the screen
    fn focus_on(&mut self, p: Point2) {
        self.offset = -p.coords + 0.5 * vector!(self.width, self.height);
    }
    fn switch(&mut self, ss: StateSwitch) {
        self.switch_state = Some(ss);
    }
}

use std::mem;

impl EventHandler for Master {
    // Handle the game logic
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(gsb) = mem::replace(&mut self.state.switch_state, None) {
            mouse::set_cursor_hidden(ctx, false);
            mouse::set_cursor_type(ctx, CursorIcon::Default);

            use self::StateSwitch::*;
            self.gs = match gsb {
                PlayWith{lvl, health, wep} => states::play::Play::new(ctx, &mut self.state, *lvl, Some((health, wep))),
                Play(lvl) => states::play::Play::new(ctx, &mut self.state, lvl, None),
                Menu => states::menu::Menu::new(ctx, &mut self.state),
                Editor(l) => states::editor::Editor::new(&self.state, l),
                Win(stats) => states::win::Win::new(ctx, &mut self.state, *stats),
                Lose(stats) => states::lose::Lose::new(ctx, &mut self.state, *stats),
            }?;
        }
        if self.console_status.is_open() {
            while ctx.time.check_update_time(DESIRED_FPS) {}

            for frag in CONSOLE_LOGGER.empty() {
                self.console.history.add(frag);
            }
            while self.console.history.dimensions(ctx).unwrap().h > PROMPT_Y {
                let new_history = self.console.history.fragments().iter().skip(1).cloned().fold(self.state.assets.raw_text(18.), |mut text, f| {
                    text.add(f);
                    text
                });
                self.console.history = new_history;
            }

            Ok(())
        } else {

            // Run this for every 1/60 of a second has passed since last update
            // Can in theory become slow
            while ctx.time.check_update_time(DESIRED_FPS) {
                self.gs.update(&mut self.state, ctx)?;
            }
            self.gs.logic(&mut self.state, ctx)
        }
    }

    // Draws everything
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Load any new assets
        self.state.assets.process_queue(ctx)?;

        // Clear the screen first
        let mut canvas = graphics::Canvas::from_frame(ctx, Some((33, 33, 255, 255).into()));

        // Save the screen coordinates and make a set with the current offset
        let sc = canvas.screen_coordinates().unwrap();
        let mut sc_offset = sc;
        sc_offset.translate(-self.state.offset);
        let sc_offset = sc_offset;

        // Draw with the offset screen coordinates
        canvas.set_screen_coordinates(sc_offset);
        self.gs.draw(&self.state, &mut canvas, ctx)?;
        
        // Restore the previous screen coordinates
        canvas.set_screen_coordinates(sc);

        self.gs.draw_hud(&self.state, &mut canvas, ctx)?;

        if self.console_status.is_open() {
            let console_bg = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(0., 0., self.state.width as f32, self.state.height as f32 / 3.), Color::BLACK)?;
            canvas.draw(&console_bg, DrawParam::new());


            canvas.draw(&self.console.history, DrawParam::default());
            self.console.prompt.draw_text(&mut canvas);
        }

        // Flip the buffers to see what we just drew
        canvas.finish(ctx)?;

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    /// Handle key down events
    fn key_down_event(&mut self, ctx: &mut Context, key_input: KeyInput, repeat: bool) -> GameResult<()> {
        // If this is a repeat event, we don't care
        if repeat {
            return Ok(())
        }

        match key_input.keycode {
            Some(VirtualKeyCode::Escape) if key_input.mods.contains(KeyMods::SHIFT) => ctx.continuing = false,
            Some(keycode) if !self.console_status.is_open() => self.gs.event_down(&mut self.state, ctx, Event::Key(keycode)),
            _ => (),
        }
        Ok(())
    }
    /// Handle key release events
    fn key_up_event(&mut self, ctx: &mut Context, key_input: KeyInput) -> GameResult<()> {
        if !self.console_status.is_open() {
            match key_input.keycode {
                Some(VirtualKeyCode::Tab) => self.console_status.open(ctx),
                Some(keycode) => self.gs.event_up(&mut self.state, ctx, Event::Key(keycode)),
                None => ()
            }
        }
        Ok(())
    }
    /// Handle mouse down event
    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) -> GameResult<()> {
        if !self.console_status.is_open() {
            self.gs.event_down(&mut self.state, ctx, Event::Mouse(btn))
        }
        Ok(())
    }
    /// Handle mouse release events
    fn mouse_button_up_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: f32, _y: f32) -> GameResult<()> {
        if !self.console_status.is_open() {
            self.gs.event_up(&mut self.state, ctx, Event::Mouse(btn))
        }
        Ok(())
    }
    fn text_input_event(&mut self, ctx: &mut Context, c: char) -> GameResult<()> {
        if self.console_status.is_open() {
            if c.is_control() {
                match c {
                    // Backspace
                    '\u{8}' => {self.console.prompt.text.fragments_mut()[1].text.pop();},
                    // Delete
                    '\u{7f}' => (),
                    // Escape
                    '\u{1b}' => self.console_status.close(ctx),
                    '\t' => {
                        // Do tab completion
                    }
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
        Ok(())
    }
    /// Handles mouse movement events
    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _: f32, _: f32) -> GameResult<()> {
        self.state.mouse = point!(x, y);
        if let ConsoleStatus::Open{cursor, cursor_hidden} = self.console_status {
            if y > PROMPT_Y {
                mouse::set_cursor_type(ctx, cursor);
                mouse::set_cursor_hidden(ctx, cursor_hidden);
            } else {
                mouse::set_cursor_type(ctx, CursorIcon::Default);
                mouse::set_cursor_hidden(ctx, false);
            }
        }
        Ok(())
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> GameResult<bool> {
        Ok(false)
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
