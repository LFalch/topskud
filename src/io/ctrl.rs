use std::collections::HashMap;

use ggez::{input::{
    mouse::{MouseButton, MouseContext},
    keyboard::{KeyCode, ScanCode, KeyInput, KeyboardContext}
}, context::Has, Context};

pub use ggez::input::keyboard::KeyMods;

#[derive(Debug, Copy, Clone)]
pub enum RawInput {
    Key {
        virtual_key_code: Option<KeyCode>,
        scan_code: Option<ScanCode>,
    },
    Button(MouseButton)
}

impl From<MouseButton> for RawInput {
    fn from(mb: MouseButton) -> Self {
        Self::Button(mb)
    }
}

impl From<KeyInput> for RawInput {
    fn from(ki: KeyInput) -> Self {
        Self::Key {
            virtual_key_code: ki.keycode,
            scan_code: Some(ki.scancode),
        }
    }
}

impl From<ScanCode> for RawInput {
    fn from(sc: ScanCode) -> Self {
        Self::Key { virtual_key_code: None, scan_code: Some(sc) }
    }
}

impl From<KeyCode> for RawInput {
    fn from(vkc: KeyCode) -> Self {
        Self::Key { virtual_key_code: Some(vkc), scan_code: None }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Controls {
    mouse_to_input: HashMap<MouseButton, Vec<Input>>,
    input_to_mouse: HashMap<Input, Vec<MouseButton>>,
    scan_to_input: HashMap<ScanCode, Vec<Input>>,
    virtual_kc_to_input: HashMap<KeyCode, Vec<Input>>,
}

const EMPTY_I: &'static Vec<Input> = &Vec::new();
const EMPTY_M: &'static Vec<MouseButton> = &Vec::new();

pub struct ControlsContext<'a> {
    controls: &'a Controls,
    pub keyboard: &'a KeyboardContext,
    pub mouse: &'a MouseContext,
}

impl ControlsContext<'_> {
    pub fn is_mod_active(&self, modifier: KeyMods) -> bool {
        self.keyboard.retrieve().is_mod_active(modifier)
    }
    pub fn is_pressed(&self, input: Input) -> bool {
        for &button in self.controls.input_to_mouse.get(&input).unwrap_or(EMPTY_M) {
            if self.mouse.button_pressed(button) {
                return true;
            }
        }
        for scancode in self.keyboard.pressed_scancodes() {
            if self.controls.scan_to_input.get(scancode).unwrap_or(EMPTY_I).contains(&input) {
                return true;
            }
        }
    
        false
    }
    pub fn axis(&self, axis: Axis) -> f32 {
        match axis {
            Axis::RightLeft => (self.is_pressed(Input::GoRight) as i8 - self.is_pressed(Input::GoLeft) as i8) as f32,
            Axis::DownUp => (self.is_pressed(Input::GoDown) as i8 - self.is_pressed(Input::GoUp) as i8) as f32,
        }
    }
}

impl Controls {
    pub fn ctx<'a>(&'a self, ctx: &'a Context) -> ControlsContext<'a> {
        self.make_context(&ctx.mouse, &ctx.keyboard)
    }
    pub fn make_context<'a>(&'a self, mouse: &'a MouseContext, keyboard: &'a KeyboardContext) -> ControlsContext<'a> {
        ControlsContext { controls: self, keyboard, mouse }
    }

    pub fn bind(&mut self, input: Input, raw_input: impl Into<RawInput>) {
        match raw_input.into() {
            RawInput::Button(mb) => {
                self.mouse_to_input.entry(mb).or_insert(Vec::new()).push(input);
                self.input_to_mouse.entry(input).or_insert(Vec::new()).push(mb);
            }
            RawInput::Key { scan_code: Some(sc), virtual_key_code: _ } => {
                self.scan_to_input.entry(sc).or_insert(Vec::new()).push(input);
            }
            RawInput::Key { virtual_key_code: Some(vkc), scan_code: None } => {
                self.virtual_kc_to_input.entry(vkc).or_insert(Vec::new()).push(input);
            }
            // RawInput::Key { virtual_key_code: Some(vkc), scan_code: Some(sc) } => {
            //     self.virtual_kc_to_input.entry(vkc).or_insert(Vec::new()).push(input);
            //     self.scan_to_input.entry(sc).or_insert(Vec::new()).push(input);
            // }
            _ => unreachable!("the raw input was empty"),
        }
    }

    // Handle events

    fn translate_key_input<'a>(scan_to_input: &'a HashMap<ScanCode, Vec<Input>>, key_input: KeyInput) -> &'a Vec<Input> {
        scan_to_input.get(&key_input.scancode).unwrap_or(EMPTY_I)
    }
    fn translate_mouse_button<'a>(mouse_to_input: &'a HashMap<MouseButton, Vec<Input>>, button: MouseButton) -> &'a Vec<Input> {
        mouse_to_input.get(&button).unwrap_or(EMPTY_I)
    }
    fn handle_key_input(&mut self, key_input: KeyInput) {
        if let Some(inputs) = key_input.keycode.and_then(|vkc| self.virtual_kc_to_input.remove(&vkc)) {
            let scancode_inputs = self.scan_to_input.entry(key_input.scancode).or_insert(Vec::new());
            
            for input in inputs {
                if !scancode_inputs.contains(&input) {
                    scancode_inputs.push(input);
                }
            }
        }
    }
    #[must_use]
    pub fn key_up(&mut self, key_input: KeyInput) -> impl Iterator<Item=Input> {
        self.handle_key_input(key_input);
        let mut events = Vec::new();
        for &input in Self::translate_key_input(&self.scan_to_input, key_input) {
            events.push(input);
        }
        events.into_iter()
    }
    #[must_use]
    pub fn key_down(&mut self, key_input: KeyInput) -> impl Iterator<Item=Input> {
        self.handle_key_input(key_input);
        let mut events = Vec::new();
        for &input in Self::translate_key_input(&self.scan_to_input, key_input) {
            events.push(input);
        }
        events.into_iter()
    }
    #[must_use]
    pub fn mouse_up(&mut self, button: MouseButton) -> impl Iterator<Item=Input> {
        let mut events = Vec::new();
        for &input in Self::translate_mouse_button(&self.mouse_to_input, button) {
            events.push(input);
        }
        events.into_iter()
    }
    #[must_use]
    pub fn mouse_down(&mut self, button: MouseButton) -> impl Iterator<Item=Input> {
        let mut events = Vec::new();
        for &input in Self::translate_mouse_button(&self.mouse_to_input, button) {
            events.push(input);
        }
        events.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Axis {
    DownUp,
    RightLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Input {
    // Movement
    GoUp,
    GoDown,
    GoLeft,
    GoRight,

    // Gameplay
    Shoot,
    ThrowGrenade,
    Reload,
    WeaponLast,
    Weapon1,
    Weapon2,
    Weapon3,
    Weapon4,
    DropWeapon,
    PickupWeapon,

    // Editor bindings
    SaveLevel,
    LoadLevel,
    ToggleVisibilityCones,
    ToggleGridSnap,
    PlayLevel,
    Deselect,
    DeleteObject,
    RotateLeft,
    RotateRight,
    MakeWaypoints,
    ToggleCyclicPath,
    DragUp,
    DragDown,
    DragLeft,
    DragRight,
    PlaceStart,

    // Misc. for menus, editor, ...
    LeftClick,
    RightClick,
    Confirm,
    Restart,
}
