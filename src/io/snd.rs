use ggez::{Context, GameResult};
use ggez::audio::{Source, SoundData};

macro_rules! ending {
    (WAV) => (".wav");
    (OGG) => (".ogg");
    (LOOP_OGG) => (".ogg");
    (FLAC) => (".flac");
}

const EFFECTS_LIMIT: usize = 25;

macro_rules! media_type {
    (WAV) => (());
    (FLAC) => (());
    (OGG) => (Source);
    (LOOP_OGG) => (Source);
}

macro_rules! play {
    (WAV, $ctx:expr, $self:expr, $snd:ident) => ({
        let src = new_source($ctx, $self.data.$snd.clone())?;
        src.play()?;

        let deads: Vec<_> =
        $self.effects.iter().enumerate().rev().filter_map(|(i, src)| if !src.playing() {
            Some(i)
        } else {None}).collect();

        for i in deads {
            $self.effects.remove(i);
        }
        if $self.effects.len() < EFFECTS_LIMIT {
            $self.effects.push(src);
        }
    });
    (FLAC, $ctx:expr, $self:expr, $snd:ident) => (
        play!(WAV, $ctx, $self, $snd);
    );
    (OGG, $ctx:expr, $self:expr, $snd:ident) => ({
        $self.$snd.play()?;
    });
    (LOOP_OGG, $ctx:expr, $self:expr, $snd:ident) => (
        play!(OGG, $ctx, $self, $snd);
    );
}

macro_rules! new_cache {
    (LOOP_OGG, $ctx:expr, $data:expr) => ({
        let mut src = Source::from_data($ctx, $data)?;
        src.set_repeat(true);
        src
    });
    (OGG, $ctx:expr, $data:expr) => (Source::from_data($ctx, $data)?);
    (FLAC, $ctx:expr, $data:expr) => (new_cache!(FLAC, $ctx, $data));
    (WAV, $ctx:expr, $data:expr) => (());
}

macro_rules! stop {
    (WAV, $ctx:expr, $self:ident, $snd:ident) => (
        unreachable!()
    );
    (FLAC, $ctx:expr, $self:ident, $snd:ident) => (
        stop!(WAV, $ctx, $self, $snd)
    );
    (OGG, $ctx:expr, $self:ident, $snd:ident) => (
        $self.$snd.stop();
    );
    (LOOP_OGG, $ctx:expr, $self:ident, $snd:ident) => ({
        stop!(OGG, $ctx, $self, $snd);
        $self.$snd = new_cache!(LOOP_OGG, $ctx, $self.data.$snd.clone());
    });
}

macro_rules! sounds {
    ($(
        $name:ident,
        $snd:ident,
        $ty:ident,
    )*) => (
        #[derive(Debug, Copy, Clone)]
        pub enum Sound {
            $($name,)*
        }

        pub struct SoundAssets {
            $(
                $snd: SoundData,
            )*
        }

        impl SoundAssets {
            pub fn new(ctx: &mut Context) -> GameResult<Self> {
                $(
                    let $snd = SoundData::new(ctx, concat!("/", stringify!($snd), ending!($ty)))?;
                )*

                Ok(SoundAssets {
                    $(
                        $snd,
                    )*
                })
            }
        }

        pub struct MediaPlayer {
            data: SoundAssets,
            effects: Vec<Source>,
            $(
                #[allow(dead_code)]
                $snd: media_type!($ty),
            )*
        }

        fn new_source(ctx: &mut Context, data: SoundData) -> GameResult<Source> {
            Source::from_data(ctx, data.clone()).map(|mut src| {
                src.set_volume(0.1);
                src
            })
        }

        impl MediaPlayer {
            pub fn new(ctx: &mut Context) -> GameResult<Self> {
                let data = SoundAssets::new(ctx)?;
                Ok(MediaPlayer {
                    effects: Vec::with_capacity(10),
                    $(
                        $snd: new_cache!($ty, ctx, data.$snd.clone()),
                    )*
                    data,
                })
            }
            pub fn play(&mut self, ctx: &mut Context, s: Sound) -> GameResult<()> {
                match s {
                    $(
                        Sound::$name => play!($ty, ctx, self, $snd),
                    )*
                }
                Ok(())
            }
            pub fn stop(&mut self, ctx: &mut Context, s: Sound) -> GameResult<()> {
                match s {
                    $(
                        Sound::$name => stop!($ty, ctx, self, $snd),
                    )*
                }
                Ok(())
            }
        }
    );
}

sounds! {
    Shot1, shot1, WAV,
    Shot2, shot2, WAV,
    Cock, cock, WAV,
    Reload, reload, WAV,
    Impact, impact, WAV,
    Hit, hit, WAV,
    Hurt, hurt, WAV,
    Death, death, WAV,
    Victory, victory, OGG,
    Music, music, LOOP_OGG,
}
