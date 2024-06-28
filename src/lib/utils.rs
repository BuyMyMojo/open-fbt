use crate::args::Args;
use crate::structs::GuildSettings;
use crate::types::Context;
use crate::types::Error;
use crate::vars::REDIS_ADDR;
use clap::Parser;
use tracing::instrument;

/// Converts a dsicord snowflake to a unix timecode
#[must_use]
pub const fn snowflake_to_unix(id: u128) -> u128 {
    const DISCORD_EPOCH: u128 = 1_420_070_400_000;

    ((id >> 22) + DISCORD_EPOCH) / 1000
}

/// Quickly checks if the verbose flag was used on launch
#[must_use]
pub fn verbose_mode() -> bool {
    let args = Args::parse();

    args.verbose
}

/// Open a tokio redis connection
#[cfg(feature = "database")]
#[instrument()]
pub async fn open_redis_connection() -> Result<redis::aio::Connection, anyhow::Error> {
    let redis_connection = redis::Client::open(REDIS_ADDR)?
        .get_tokio_connection()
        .await?;

    Ok(redis_connection)
}

/// Pushes guild settings to DB
#[cfg(feature = "database")]
#[instrument(skip(con))]
pub async fn set_guild_settings(
    ctx: Context<'_>,
    con: &mut redis::aio::Connection,
    settings: GuildSettings,
) -> Result<(), Error> {
    let json = serde_json::to_string(&settings).unwrap();

    let mut pipe = redis::pipe();

    pipe.cmd("JSON.SET").arg(&[
        format!(
            "guild-settings:{}",
            ctx.guild_id().expect("Not run inside guild")
        ),
        "$".to_string(),
        json,
    ]);

    pipe.atomic().query_async(con).await?;

    Ok(())
}

/// Adds the user to a server's auth list in the DB
#[cfg(feature = "database")]
#[instrument(skip(con))]
pub async fn auth(
    ctx: Context<'_>,
    con: &mut redis::aio::Connection,
    uid: String,
) -> Result<(), Error> {
    redis::cmd("SADD")
        .arg(&[
            format!(
                "authed-server-users:{}",
                ctx.guild_id().expect("Not run inside guild")
            ),
            uid,
        ])
        .query_async(con)
        .await?;

    Ok(())
}

/// Increases the total commands run count in the DB
#[cfg(feature = "database")]
#[instrument]
pub async fn inc_execution_count() -> Result<(), Error> {
    let mut con = open_redis_connection().await?;

    // increment status:commands-executed in redis DB
    redis::cmd("INCR")
        .arg("status:commands-executed")
        .query_async(&mut con)
        .await?;

    Ok(())
}

#[cfg(feature = "database")]
#[instrument]
pub async fn is_uid_valid_user(uid: u64, ctx: &Context<'_>) -> anyhow::Result<bool> {
    let u_opt: Option<poise::serenity_prelude::User> =
        match poise::serenity_prelude::UserId::from(uid)
            .to_user(ctx)
            .await
        {
            Ok(user) => Some(user),
            Err(error) => {
                if verbose_mode() {
                    ctx.say(format!(
                        "ID must be a user ID, make sure you coppied the right one! Error: {:?}",
                        error
                    ))
                    .await?;
                } else {
                    ctx.say("ID must be a user ID, make sure you coppied the right one!")
                        .await?;
                }

                None
            }
        };

    Ok(u_opt.is_some())
}

#[cfg(test)]
mod utils_tests {

    use crate::utils::{snowflake_to_unix, verbose_mode};

    #[test]
    fn snowflake_unix_test() {
        assert_eq!(snowflake_to_unix(383_507_911_160_233_985), 1_511_505_811);
    }

    #[test]
    fn verbose_mode_test() {
        // Inverting output since verbose mode is disabled by default
        assert!(!verbose_mode());
    }
}
