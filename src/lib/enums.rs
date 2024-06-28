use poise::serenity_prelude::{self as serenity};

#[derive(Debug, poise::ChoiceParameter)]
pub enum WaifuTypes {
    Neko,
    Megumin,
    Bully,
    Cuddle,
    Cry,
    Kiss,
    Lick,
    Pat,
    Smug,
    Bonk,
    Blush,
    Smile,
    Wave,
    Highfive,
    Handhold,
    Nom,
    Bite,
    Glomp,
    Slap,
    Kill,
    Happy,
    Wink,
    Poke,
    Dance,
    Cringe,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum)]
pub enum DebugLevel {
    Off,
    Some,
    Most,
    All,
}

impl DebugLevel {
    #[must_use]
    pub fn enabled(&self) -> bool {
        *self != Self::Off
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum)]
pub enum LogDebugLevel {
    Most,
    All,
}

pub enum CloseTicketFail {
    False,
    IncorrectCategory,
    SerenityError(serenity::Error),
}
