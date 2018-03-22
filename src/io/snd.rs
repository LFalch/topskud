use ggez::{Context, GameResult};
use ggez::audio::{Source, SoundData};

macro_rules! ending {
    (WAV) => (".wav");
    (OGG) => (".ogg");
    (FLAC) => (".flac");
}

macro_rules! sounds {
    ($(
        $name:ident,
        $snd:ident,
        $ty:ident,
    )*) => (
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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

                Ok((SoundAssets {
                    $(
                        $snd,
                    )*
                }))
            }
            pub fn make_source(&self, ctx: &mut Context, s: Sound) -> GameResult<Source> {
                let data = match s {
                    $(
                        Sound::$name => &self.$snd,
                    )*
                };
                Source::from_data(ctx, data.clone()).map(|mut src| {
                    src.set_volume(0.1);
                    src
                })
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
}
