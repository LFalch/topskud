use std::collections::HashMap;
use super::{Console, State, GameState, Command, CommandError, Content, StateSwitch};

use crate::{
    util::dbg_strs,
    obj::{health::Health, weapon::WEAPONS},
};
use ggez::Context;

macro_rules! commands {
    ($console:ident, $ctx:ident, $state:ident, $gs:ident, $args:ident, $(
        $($name:expr),+ => $f:block,
    )*) => {
        let mut map = HashMap::new();

        $({
            #[allow(unused_variables)]
            fn function<'a>($console: &'a mut Console, $ctx: &'a mut Context, $state: &'a mut State, $gs: &'a mut dyn GameState, $args: Vec<&'a str>) -> Result<(), CommandError> $f

            let cmd = function as Command;
            
            $(
                map.insert($name.into(), cmd);
            )+
        })*

        map
    };
}

pub(super) fn commands() -> HashMap<String, Command> {
    use CommandError::*;

    commands!{console, ctx, state, gs, args,
        "" => {Ok(())},
        "pi", "intels" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            world.intels.clear();
            info!("Intels got");
            Ok(())
        },
        "clear", "cl" => {
            console.history = state.assets.raw_text_with("", 18.);
            Ok(())
        },
        "fa", "fullarmour" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            world.player.health.hp = 100.;
            world.player.health.armour = 100.;
            Ok(())
        },
        "god" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            if world.player.health.hp.is_finite() {
                world.player.health.hp = std::f32::INFINITY;
                info!("Degreelessness");
            } else {
                world.player.health.hp = 100.;
                info!("God off");
            }
            Ok(())
        },
        "godarmour", "ga" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            if world.player.health.armour.is_finite() {
                world.player.health.armour = std::f32::INFINITY;
                info!("Skin of steel");
            } else {
                world.player.health.armour = 5.;
                info!("Re-skin");
            }
            Ok(())
        },
        "wep" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            let &wep = args.get(1).ok_or(InvalidArg)?;
            let weapon = WEAPONS.get(wep).ok_or(NoSuchWeapon)?;

            world.weapons.push(weapon.make_drop(state.mouse-state.offset));
            Ok(())
        },
        "reload" => {
            if let Some(arg) = args.get(1) {
                state.content = Content::File(arg.to_owned().into())
            } else {
                state.content = Content::None
            }
            state.switch(StateSwitch::Menu);
            Ok(())
        },
        "cmp" => {if let Content::Campaign(ref mut cmp) = state.content {
            if let Some(i) = args.get(1) {
                let i = i.parse().map_err(|_| InvalidArg)?;
                cmp.current = i;
                let lvl = cmp.next_level().ok_or(NoSuchLevel)?;

                let (health, wep) = if let Some(world) = gs.get_world() {
                    (world.player.health, world.player.wep.clone())
                } else {
                    (Health::default(), Default::default())
                };

                state.switch(StateSwitch::PlayWith{health, wep, lvl: Box::new(lvl)});
            } else {
                info!("{} levels. Current is {}", cmp.levels.len(), cmp.current);
            }
            Ok(())
        } else {
            Err(NoCampaign)
        }},
        "gg" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            world.player.utilities.grenades += 3;
            info!("Gg'd");
            Ok(())
        },
        "hello" => {
            info!("Hello!");
            Ok(())
        },
        "quit" => {
            ctx.continuing = false;
            Ok(())
        },
        "strs" => {
            dbg_strs();
            Ok(())
        },
    }
}