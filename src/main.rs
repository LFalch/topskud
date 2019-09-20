// #![windows_subsystem = "windows"]
//! Shooter game
#![warn(clippy::all)]

#[macro_use]
extern crate serde_derive;

use std::env::args;

use ggez::{
    ContextBuilder,
    conf,
    filesystem,
    event::run,
};

pub mod io;
pub mod obj;
pub mod ext;
pub mod game;

pub mod util {
    use ggez::graphics::Color;
    pub type Vector2 = nalgebra::Vector2<f32>;
    pub type Point2 = nalgebra::Point2<f32>;

    pub const TRANS: Color = Color{r:1.,g:1.,b:1.,a:0.5};
    pub const GREEN: Color = Color{r:0.1,g:0.7,b:0.1,a:1.};
    pub const RED: Color = Color{r:1.,g:0.,b:0.,a:1.};
    pub const BLUE: Color = Color{r:0.,g:0.,b:1.,a:1.};

    /// Makes a unit vector from a given direction angle
    pub fn angle_to_vec(angle: f32) -> Vector2 {
        let (sin, cos) = angle.sin_cos();
        Vector2::new(cos, sin)
    }
    /// Gets the direction angle on the screen (0 is along the x-axis) of a vector
    pub fn angle_from_vec(v: Vector2) -> f32 {
        let x = v.x;
        let y = v.y;

        y.atan2(x)
    }
}

use self::game::Master;

fn main() {
    let mut args = args().skip(1);

    let arg;
    if let Some(p) = args.next() {
        arg = p;
    } else {
        arg = "".to_owned();
    };

    // Set window mode
    let window_mode = conf::WindowMode::default().dimensions(1152., 648.);

    // Create a context (the part that runs the game loop)
    let (mut ctx, mut events) = ContextBuilder::new("tds", "LFalch")
        .window_setup(conf::WindowSetup::default().title("TDS"))
        .window_mode(window_mode)
        .build().unwrap();

    // Add the workspace directory to the filesystem when running with cargo
    // This is only used in development
    if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = ::std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        filesystem::mount(&mut ctx, &path, true);
    }

    // Tries to create a game state and runs it if succesful
    match Master::new(&mut ctx, &arg) {
        Err(e) => {
            eprintln!("Couldn't load game {}", e);
        }
        Ok(mut game) => {
            // Run the game loop
            match run(&mut ctx, &mut events, &mut game) {
                Ok(_) => (),
                Err(e) => eprintln!("Error occured: {}", e)
            }
        }
    }
}
