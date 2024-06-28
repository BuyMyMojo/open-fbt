use crate::enums::{DebugLevel, LogDebugLevel};
use crate::vars::BOT_TOKEN;
use clap::Parser;

/// This is where CLI args are set
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Discord bot token
    #[clap(
        short,
        long,
        default_value = BOT_TOKEN
    )]
    pub token: String,

    /// Command prefix for message commands
    #[clap(short, long, default_value = "`")]
    pub prefix: String,

    /// Output extra information in discord reply errors
    #[clap(short, long)]
    pub verbose: bool,

    /// Print out list of guilds in cache on startup
    #[clap(long("gpc"))]
    pub print_guild_cache: bool,

    /// emit debug information to both stdout and a file
    #[clap(value_enum, long, default_value = "most")]
    pub debug: DebugLevel,

    /// emit debug information to both stdout and a file
    #[clap(value_enum, long, default_value = "most")]
    pub debug_log: LogDebugLevel,
}
