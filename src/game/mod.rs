use io::snd::MediaPlayer;
use ::*;

use std::path::PathBuf;

/// Stuff related to things in the world
pub mod world;
pub mod editor;
pub mod play;
pub mod menu;

use menu::Menu;

pub enum StateSwitch {
    Menu,
    Editor,
    Play,
}

pub trait GameState {
    fn update(&mut self, &mut State, &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn logic(&mut self, &mut State, &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, &State, &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw_hud(&mut self, &State, &mut Context) -> GameResult<()>;

    fn key_down(&mut self, &mut State, &mut Context, Keycode) {

    }
    fn key_up(&mut self, &mut State, &mut Context, Keycode) {

    }
    fn mouse_down(&mut self, &mut State, &mut Context, MouseButton) {

    }
    fn mouse_up(&mut self, &mut State, &mut Context, MouseButton) {

    }
}

pub struct Master {
    gs: Box<GameState>,
    state: State,
}

/// The state of the game
pub struct State {
    mouse_down: MouseDown,
    input: InputState,
    modifiers: Modifiers,
    assets: Assets,
    mplayer: MediaPlayer,
    width: u32,
    height: u32,
    mouse: Point2,
    offset: Vector2,
    switch_state: Option<StateSwitch>,
    save: PathBuf,
    level: Option<Level>,
}

const DESIRED_FPS: u32 = 60;

pub(crate) const DELTA: f32 = 1. / DESIRED_FPS as f32;

impl Master {
    /// Make a new state object
    pub fn new(ctx: &mut Context, p: &str, level: Option<Level>) -> GameResult<Self> {
        // Background colour is black
        graphics::set_background_color(ctx, (33, 33, 255, 255).into());
        // Initialise assets
        let assets = Assets::new(ctx)?;
        let mplayer = MediaPlayer::new(ctx)?;

        // Get the window's dimensions
        let width = ctx.conf.window_mode.width;
        let height = ctx.conf.window_mode.height;

        let mut state = State {
            save: p.to_owned().into(),
            level,
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
            gs: Menu::new(ctx, &mut state)?,
            state,
        })
    }
}

impl State {
    /// Sets the offset so that the given point will be centered on the screen
    fn focus_on(&mut self, p: Point2) {
        self.offset = -p.coords + 0.5 * Vector2::new(self.width as f32, self.height as f32);
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
            use StateSwitch::*;
            self.gs = match gsb {
                Play => play::Play::new(ctx, &mut self.state),
                Menu => menu::Menu::new(ctx, &mut self.state),
                Editor => editor::Editor::new(ctx, &self.state),
            }?;
        }

        // Run this for every 1/60 of a second has passed since last update
        // Can in theory become slow
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.gs.update(&mut self.state, ctx)?;
        }
        self.gs.logic(&mut self.state, ctx)
    }

    // Draws everything
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Clear the screen first
        graphics::clear(ctx);

        // Offset the current drawing with a translation from the `offset`
        graphics::push_transform(ctx, Some(Matrix4::new_translation(&self.state.offset.fixed_resize(0.))));
        graphics::apply_transformations(ctx)?;

        self.gs.draw(&self.state, ctx)?;

        // Pop the offset tranformation to draw the UI on the screen
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;

        self.gs.draw_hud(&self.state, ctx)?;

        // Flip the buffers to see what we just drew
        graphics::present(ctx);

        // Give the computer some time to do other things
        timer::yield_now();
        Ok(())
    }
    /// Handle key down events
    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _: Mod, repeat: bool) {
        // If this is a repeat event, we don't care
        if repeat {
            return
        }

        use Keycode::*;
        // Update input axes and quit game on Escape
        match keycode {
            W | Up => self.state.input.ver -= 1,
            S | Down => self.state.input.ver += 1,
            A | Left => self.state.input.hor -= 1,
            D | Right => self.state.input.hor += 1,
            LShift => self.state.modifiers.shift = true,
            LCtrl => self.state.modifiers.ctrl = true,
            LAlt => self.state.modifiers.alt = true,
            Escape => ctx.quit().unwrap(),
            _ => (),
        }
        self.gs.key_down(&mut self.state, ctx, keycode)
    }
    /// Handle key release events
    fn key_up_event(&mut self, ctx: &mut Context, keycode: Keycode, _: Mod, repeat: bool) {
        // Still don't care about repeats
        if repeat {
            return
        }
        use Keycode::*;

        match keycode {
            W | Up => self.state.input.ver += 1,
            S | Down => self.state.input.ver -= 1,
            A | Left => self.state.input.hor += 1,
            D | Right => self.state.input.hor -= 1,
            LShift => self.state.modifiers.shift = false,
            LCtrl => self.state.modifiers.ctrl = false,
            LAlt => self.state.modifiers.alt = false,
            _ => (),
        }
        self.gs.key_up(&mut self.state, ctx, keycode)
    }
    /// Handle mouse down event
    fn mouse_button_down_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: i32, _y: i32) {
        use MouseButton::*;
        match btn {
            Left => self.state.mouse_down.left = true,
            Middle => self.state.mouse_down.middle = true,
            Right => self.state.mouse_down.right = true,
            _ => ()
        }
        self.gs.mouse_down(&mut self.state, ctx, btn)
    }
    /// Handle mouse release events
    fn mouse_button_up_event(&mut self, ctx: &mut Context, btn: MouseButton, _x: i32, _y: i32) {
        use MouseButton::*;
        match btn {
            Left => self.state.mouse_down.left = false,
            Middle => self.state.mouse_down.middle = false,
            Right => self.state.mouse_down.right = false,
            _ => ()
        }
        self.gs.mouse_up(&mut self.state, ctx, btn)
    }
    /// Handles mouse movement events
    fn mouse_motion_event(&mut self, _: &mut Context, _: MouseState, x: i32, y: i32, _: i32, _: i32) {
        self.state.mouse = Point2::new(x as f32, y as f32);
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        false
    }
}
