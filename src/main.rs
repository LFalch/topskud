// #![windows_subsystem = "windows"]
//! Shooter game
#![warn(clippy::all)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nalgebra;

use std::env::args;

use ggez::{
    ContextBuilder,
    conf,
    event::run,
};

pub mod game;

use self::game::Master;
 
fn main() {
    let mut args = args().skip(1);
    let arg = args.next().unwrap_or_default();

    // Set window mode
    let window_mode = conf::WindowMode::default().dimensions(1152., 648.);

    // Create a context (the part that runs the game loop)
    let (mut ctx, events) = ContextBuilder::new("topskud", "LFalch")
        .window_setup(conf::WindowSetup::default().title("Topskud"))
        .window_mode(window_mode)
        .build().unwrap();

    #[cfg(debug_assertions)]
    {
        // Add the workspace directory to the filesystem when running with cargo
        if let Ok(manifest_dir) = ::std::env::var("CARGO_MANIFEST_DIR") {
            let mut path = ::std::path::PathBuf::from(manifest_dir);
            path.push("resources");
            ctx.fs.mount(&path, true);
        }
    }

    match Master::new(&mut ctx, &arg) {
        Err(e) => {
            eprintln!("Couldn't load game {}", e);
        }
        Ok(game) => run(ctx, events, game),
    }
}
