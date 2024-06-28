use crate::types::{Context, Error};
use crate::vars::BOT_ADMINS;
use std::collections::HashSet;

/// Check if command user is in the `BOT_ADMINS` list
///
/// # Errors
///
/// This function will never return an error.
#[allow(clippy::unused_async, clippy::missing_errors_doc)] // async is used by command checks but clippy can't tell
pub async fn bot_admin_check(ctx: Context<'_>) -> Result<bool, Error> {
    // ? The bellow commented out code is for quick testing, automatic fails on my ID
    // match ctx.author().id.as_u64() {
    //     383507911160233985 => Ok(false),
    //     _ => {
    //         match BOT_ADMINS.contains(ctx.author().id.as_u64()) {
    //             true => Ok(true),
    //             false => Ok(false),
    //         }
    //     }
    // }
    if BOT_ADMINS.contains(ctx.author().id.as_u64()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

// ? This might not be needed, I thinik it's a left over from before we dud guild based authing
// ! Remove the _ if put into use!
#[cfg(feature = "database")]
#[deprecated(
    since = "0.1.12",
    note = "left over from before we dud guild based auth"
)]
#[allow(clippy::unused_async, clippy::missing_errors_doc)] // no need to lint dead code
pub async fn _bot_auth_check(_ctx: Context<'_>) -> Result<bool, Error> {
    // if let Ok(res) = bot_admin_check(ctx).await {
    //     if res {
    //         return Ok(true);
    //     }
    // }

    // let mut con = open_redis_connection().await?;

    // let key_list: HashSet<u64> = redis::cmd("SMEMBERS")
    //     .arg("user-lists:authorised-users")
    //     .clone()
    //     .query_async(&mut con)
    //     .await?;

    // if key_list.contains(ctx.author().id.as_u64()) {
    //     Ok(true)
    // } else {
    //     ctx.say("You are not authorized to use this command! Please contact a bot admin or Azuki!")
    //         .await?;
    //     Ok(false)
    // }
    Ok(false)
}

/// Checks if a user is authorised to use the bot in the current server
///
/// # Errors
///
/// This function will return an error if unable to connet to or query DB.
#[cfg(feature = "database")]
pub async fn guild_auth_check(ctx: Context<'_>) -> Result<bool, Error> {
    use crate::utils::open_redis_connection;

    if let Ok(res) = bot_admin_check(ctx).await {
        if res {
            return Ok(true);
        }
    }

    let mut con = open_redis_connection().await?;

    let key_list: Option<HashSet<String>> = redis::cmd("SMEMBERS")
        .arg(format!(
            "authed-server-users:{}",
            ctx.guild_id()
                .unwrap_or(poise::serenity_prelude::GuildId(0))
                .as_u64()
        ))
        .clone()
        .query_async(&mut con)
        .await?;

    match key_list {
        None => Ok(false),
        Some(list) if !list.contains(&format!("{}", ctx.author().id.as_u64())) => {
            Ok({
                ctx.say("You are not authorized to use this command! Please contact a bot admin or Azuki!").await?;
                false
            })
        }
        Some(_list) => Ok(true),
    }
}
