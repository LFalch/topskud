use std::collections::HashMap;

use ggez::audio::{SoundData, Source};
use ggez::{Context, GameResult};

const EFFECTS_LIMIT: usize = 25;

#[inline]
fn new_source(ctx: &mut Context, data: &SoundData) -> GameResult<Source> {
    Source::from_data(ctx, data.clone()).map(|mut src| {
        src.set_volume(0.1);
        src
    })
}

#[derive(Debug, Copy, Clone)]
enum SoundType {
    Wave,
    #[allow(dead_code)]
    Flac,
    Ogg,
    OggLoop,
}

impl SoundType {
    #[inline]
    fn is_loop(self) -> bool {
        match self {
            SoundType::OggLoop => true,
            _ => false,
        }
    }
}
macro_rules! ending {
    (Wave) => {
        ".wav"
    };
    (Ogg) => {
        ".ogg"
    };
    (OggLoop) => {
        ".ogg"
    };
    (Flac) => {
        ".flac"
    };
}

macro_rules! music {
    (Wave; $b:block) => (());
    (Flac; $b:block) => (());
    (Ogg; $b:block) => ($b);
    (OggLoop; $b:block) => (music!(Ogg; $b));
}

macro_rules! sounds {
    ($(
        $name:ident,
        $snd:expr,
        $typ:ident,
    )*) => (
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Sound {
            $($name,)*
        }
        impl Sound {
            #[inline]
            fn sound_type(self) -> SoundType {
                match self {
                    $(
                        Sound::$name => SoundType::$typ,
                    )*
                }
            }
        }
        pub struct MediaPlayer {
            data: HashMap<Sound, SoundData>,
            music_sources: HashMap<Sound, Source>,
            effects: Vec<Source>,
        }
        impl MediaPlayer {
            #[allow(clippy::new_ret_no_self)]
            pub fn new(ctx: &mut Context) -> GameResult<Self> {
                let mut data = HashMap::new();
                $(
                    data.insert($name, SoundData::new(ctx, concat!("/sounds/", $snd, ending!($typ)))?);
                )*
                let mut ret = MediaPlayer {
                    effects: Vec::with_capacity(10),
                    music_sources: HashMap::new(),
                    data,
                };
                $(
                    music!($typ; {
                        ret.music_sources.insert($name, ret.new_cache(ctx, Sound::$name, Sound::$name.sound_type().is_loop())?);
                    });
                )*

                Ok(ret)
            }
            pub fn play(&mut self, ctx: &mut Context, s: Sound) -> GameResult<()> {
                match s.sound_type() {
                    SoundType::Wave | SoundType::Flac => {
                        let src = new_source(ctx, &self.data[&s])?;
                        src.play()?;

                        self.clear_effetcs();

                        if self.effects.len() < EFFECTS_LIMIT {
                            self.effects.push(src);
                        }
                        Ok(())
                    },
                    SoundType::Ogg | SoundType::OggLoop => {
                        self.music_sources[&s].play()
                    },
                }
            }
            fn clear_effetcs(&mut self) {
                let deads: Vec<_> =
                self.effects.iter().enumerate().rev().filter_map(|(i, src)| if !src.playing() {
                    Some(i)
                } else {None}).collect();

                for i in deads {
                    self.effects.remove(i);
                }
            }
            fn new_cache(&self, ctx: &mut Context, s: Sound, repeat: bool) -> GameResult<Source> {
                Source::from_data(ctx, self.data[&s].clone())
                    .map(|mut src| {
                        src.set_repeat(repeat);
                        src
                    })
            }
            pub fn stop(&mut self, ctx: &mut Context, s: Sound) -> GameResult<()> {
                match s.sound_type() {
                    SoundType::Wave | SoundType::Flac => panic!("{:?} can't be stopped", s),
                    SoundType::Ogg => self.music_sources[&s].stop(),
                    SoundType::OggLoop => {
                        self.music_sources[&s].stop();
                        self.music_sources.insert(s, self.new_cache(ctx, s, true)?);
                    }
                }
                Ok(())
            }
        }
    );
}

use self::Sound::*;

sounds! {
    Shot1, "shot1", Wave,
    Shot2, "shot2", Wave,
    Cock, "cock", Wave,
    Cock2, "cock2", Wave,
    CockAk47, "cock_ak47", Wave,
    Reload, "reload", Wave,
    ReloadM4, "reload_m4", Wave,
    ReloadAk47, "reload_ak47", Wave,
    ClickPistol, "click_pistol", Wave,
    ClickUzi, "click_uzi", Wave,
    Impact, "impact", Wave,
    Hit, "hit", Wave,
    Hurt, "hurt", Wave,
    Death, "death", Wave,
    Victory, "victory", Ogg,
    Music, "music", OggLoop,
}
