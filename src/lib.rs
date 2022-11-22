// #![windows_subsystem = "windows"]
//! Shooter game
#![warn(clippy::all)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nalgebra;

pub mod io;
pub mod obj;
pub mod ext;
pub mod world;

pub const DESIRED_FPS: u32 = 60;

pub const DELTA: f32 = 1. / DESIRED_FPS as f32;

pub mod util {
    use std::{collections::HashSet, sync::Mutex};
    use ggez::GameResult;
    use lazy_static::lazy_static;
    use ggez::graphics::Color;
    use serde::{Deserializer, Deserialize};
    use nalgebra::base::coordinates::XY;
    pub type Vector2 = nalgebra::Vector2<f32>;
    pub type Point2 = nalgebra::Point2<f32>;
    pub type Rotation2 = nalgebra::Rotation2<f32>;

    pub const TRANS: Color = Color{r:1.,g:1.,b:1.,a:0.5};
    pub const GREEN: Color = Color{r:0.1,g:0.7,b:0.1,a:1.};
    pub const RED: Color = Color{r:1.,g:0.,b:0.,a:1.};
    pub const BLUE: Color = Color{r:0.,g:0.,b:1.,a:1.};

    /// Makes a unit vector from a given direction angle
    pub fn angle_to_vec(angle: f32) -> Vector2 {
        let (sin, cos) = angle.sin_cos();
        vector!(cos, sin)
    }
    /// Gets the direction angle on the screen (0 is along the x-axis) of a vector
    pub fn angle_from_vec(v: Vector2) -> f32 {
        let XY{x, y} = *v;
        y.atan2(x)
    }

    lazy_static! {
        static ref STATIC_STRINGS: Mutex<HashSet<Sstr>> = Mutex::new(HashSet::new());
    }

    // A static string
    pub type Sstr = &'static str; 

    /// Gives you a reference to a static slice with the contents of the given string.
    /// If it isn't already in the static strings list, a new one will be created from a `Box`.
    pub fn sstr<S: AsRef<str> + Into<Box<str>>>(s: S) -> Sstr {
        let mut lock = STATIC_STRINGS.lock().unwrap();

        if !lock.contains(s.as_ref()) {
            let s = &*Box::leak(s.into());
            lock.insert(s);
            s
        } else {
            lock.get(s.as_ref()).unwrap()
        }
    }
    #[inline]
    pub fn add_sstr(s: Sstr) -> Sstr {
        let mut lock = STATIC_STRINGS.lock().unwrap();

        if !lock.contains(s) {
            lock.insert(s);
        }
        s
    }
    #[inline]
    pub fn deserialize_sstr<'de, D: Deserializer<'de>>(d: D) -> Result<Sstr, D::Error> {
        <Box<str>>::deserialize(d).map(sstr)
    }
    pub fn dbg_strs() {
        let lock = STATIC_STRINGS.lock().unwrap();

        info!("{:?}", *lock);
    }

    /// For each element where `f` returns `true`, it will be removed from the vector
    pub fn iterate_and_kill_one<T, F: for<'a> FnMut(&'a T) -> bool>(vec: &mut Vec<T>, mut f: F) -> Option<T> {
        let mut dead = None;

        for (i, element) in vec.iter().enumerate() {
            if f(element) {
                dead = Some(i);
                break;
            }
        }

        dead.map(|i| vec.remove(i))
    }
    /// For each element where `f` returns `true`, it will be removed from the vector
    pub fn iterate_and_kill_one_mut<T, F: for<'a> FnMut(&'a mut T) -> bool>(vec: &mut Vec<T>, mut f: F) -> Option<T> {
        let mut dead = None;

        for (i, element) in vec.iter_mut().enumerate() {
            if f(element) {
                dead = Some(i);
                break;
            }
        }

        dead.map(|i| vec.remove(i))
    }
    /// For each element where `f` returns `true`, it will be removed from the vector
    pub fn iterate_and_kill_afterwards<T, F: for<'a> FnMut(&'a T) -> GameResult<bool>>(vec: &mut Vec<T>, mut f: F) -> GameResult<()> {
        let mut deads = Vec::new();

        for (i, element) in vec.iter().enumerate().rev() {
            if f(element)? {
                deads.push(i);
            }
        }

        for index in deads.into_iter() {
            vec.remove(index);
        }

        Ok(())
    }
    /// For each element where `f` returns `true`, it will be removed from the vector
    pub fn iterate_and_kill_afterwards_mut<T, F: for<'a> FnMut(&'a mut T) -> GameResult<bool>>(vec: &mut Vec<T>, mut f: F) -> GameResult<()> {
        let mut deads = Vec::new();

        for (i, element) in vec.iter_mut().enumerate().rev() {
            if f(element)? {
                deads.push(i);
            }
        }

        for index in deads.into_iter() {
            vec.remove(index);
        }

        Ok(())
    }
}
