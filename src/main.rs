// #![windows_subsystem = "windows"]
//! Shooter game

extern crate ggez;
extern crate bincode;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate self_compare;
extern crate rand;

use std::env::args;

use ggez::conf;
use ggez::event::*;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::timer;
use ggez::graphics::{self, Vector2, Point2, Matrix4, Color};

mod io;
pub use io::tex::*;

mod obj;
pub use obj::*;
mod ext;
pub use ext::*;
mod game;
pub use game::*;

use game::world::Level;

/// Makes a unit vector from a given direction angle
fn angle_to_vec(angle: f32) -> Vector2 {
    let (sin, cos) = angle.sin_cos();
    Vector2::new(cos, sin)
}
/// Gets the direction angle on the screen (0 is along the x-axis) of a vector
pub fn angle_from_vec(v: &Vector2) -> f32 {
    let x = v.x;
    let y = v.y;

    y.atan2(x)
}

pub const TRANS: Color = Color{r:1.,g:1.,b:1.,a:0.5};
pub const GREEN: Color = Color{r:0.,g:1.,b:0.,a:1.};
pub const RED: Color = Color{r:1.,g:0.,b:0.,a:1.};
pub const BLUE: Color = Color{r:0.,g:0.,b:1.,a:1.};

fn main() {
    let mut args = args().skip(1);

    let mut level = None;
    let content = if let Some(mut p) = args.next() {
        if &p == "--new" {
            p = args.next().unwrap();
            let w: u16 = args.next().unwrap().parse().unwrap();
            let h: u16 = args.next().unwrap().parse().unwrap();

            level = Some(Level::new(w, h));
        }
        Content::File(p.to_owned().into())
    } else {
        Content::Campaign(Campaign::load("start.cmp").unwrap())
    };

    // Set window mode
    let window_mode = conf::WindowMode::default().dimensions(1152, 648);

    // Create a context (the part that runs the game loop)
    let mut ctx = ContextBuilder::new("tds", "LFalch")
        .window_setup(conf::WindowSetup::default().title("TDS"))
        .window_mode(window_mode)
        .build().unwrap();

    // Add the workspace directory to the filesystem when running with cargo
    // This is only used in development
    if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = ::std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    // Tries to create a game state and runs it if succesful
    match Master::new(&mut ctx, content, level) {
        Err(e) => {
            eprintln!("Couldn't load game {}", e);
        }
        Ok(mut game) => {
            // Run the game loop
            match run(&mut ctx, &mut game) {
                Ok(_) => (),
                Err(e) => eprintln!("Error occured: {}", e)
            }
        }
    }
}
