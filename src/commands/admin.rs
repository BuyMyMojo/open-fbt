use clap::Parser;
use core::time;
use poise::serenity_prelude::Attachment;
use poise::serenity_prelude::{self as serenity, Activity, Member, OnlineStatus};
use poise::serenity_prelude::{ChannelId, Colour};
use rand::seq::SliceRandom;
use rand::Rng;
use rusted_fbt_lib::checks::guild_auth_check;
use rusted_fbt_lib::structs::GuildSettings;
use rusted_fbt_lib::utils::{auth, open_redis_connection, set_guild_settings};
use rusted_fbt_lib::{
    args::Args,
    checks::bot_admin_check,
    types::{Context, Error},
    utils::{inc_execution_count, verbose_mode},
};
use std::collections::HashSet;
use std::ops::Add;
use std::process::exit;
use tokio::time::sleep;
use tracing::instrument;
use tracing::{event, Level};

#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    category = "Admin",
    check = "bot_admin_check",
    hide_in_help
)]
/// Sends message to specified user ID
pub async fn botmsg(
    ctx: Context<'_>,
    #[description = "User ID"] user: serenity::User,
    #[description = "Message"] msg: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    user.direct_message(ctx, |f| f.content(&msg)).await?;

    ctx.say(format!("Sent message to: {}", user.name)).await?;
    ctx.say(format!("Message: {}", &msg)).await?;

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    guild_only,
    ephemeral
)]
/// Explains how to ban a list of users with `ban
pub async fn ban_help(ctx: Context<'_>) -> Result<(), Error> {
    let args = Args::parse();

    ctx.say("To ban a single user the easiest way is with the slash command `/ban ban_user @USER/ID` since you don't need to provide message deletion numbers or a reason.").await?;
    ctx.say(format!("In order to ban multiple people please use this command as a a prefix command like so:\n```\n{}ban ban_user \"Reason in qutoation marks\" 0(A number from 0 to 7, how many days worth of messages you want to delet) userID1 userID2 userID3\n```", args.prefix)).await?;

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    guild_only,
    ephemeral
)]
/// Explains how to ban a list of users with `ban
pub async fn ban_user(
    ctx: Context<'_>,
    #[description = "Ban reason"] reason: Option<String>,
    #[description = "How many days of messages to purge (Max of 7)"]
    #[min = 0]
    #[max = 7]
    dmd: Option<u8>,
    #[description = "Member(s) to ban"] members: Vec<Member>,
) -> Result<(), Error> {
    let delete_count: u8 = dmd.map_or(0u8, |num| {
        let num_check = num;
        if num_check.le(&7u8) {
            num
        } else {
            7u8
        }
    });

    let reason_sanitised = reason.map_or_else(|| "Banned via bot ban command".to_string(), |r| r);

    // TODO: Change to your own emojis!

    // Mojo test server emoji version
    // let phrase_list = vec!["has been ejected", "banned quietly", "terminated", "thrown out", "<a:Banned1:1000474420864880831><a:Banned2:1000474423683452998><a:Banned3:1000474426447503441>"];

    // FBT Emoji version
    let phrase_list = ["has been ejected", "banned quietly", "terminated", "thrown out", "<a:Banned1:1000474106929631433><a:Banned2:1000474109802725457><a:Banned3:1000474112734531715>"];

    match ctx.guild() {
        Some(guild) => match members.len() {
            0 => {
                let args = Args::parse();

                ctx.say("You must provide at least one user to ban!")
                    .await?;
                ctx.say(format!("In order to ban multiple people please use this command as a a prefix command like so:\n```\n{}ban userID1 userID2 userID3\n```", args.prefix)).await?;
            }
            1 => {
                let member = guild.member(ctx, members[0].user.id).await?;
                if let Err(error) = member
                    .ban_with_reason(ctx, delete_count, reason_sanitised.clone())
                    .await
                {
                    if verbose_mode() {
                        ctx.say(format!(
                            "Failed to ban {} because of {:?}",
                            member.display_name(),
                            error
                        ))
                        .await?;
                    } else {
                        ctx.say(format!("Failed to ban {}", member.display_name()))
                            .await?;
                    }
                    // 0u8
                } else {
                    let phrase = phrase_list
                        .choose(&mut rand::thread_rng())
                        .expect("Unable to get meme phrase for ban");
                    ctx.say(format!("{} has been {phrase}", member.display_name()))
                        .await?;
                    // 0u8
                };
            }
            _ => {
                for member in members {
                    if let Err(error) = member
                        .ban_with_reason(ctx, delete_count, reason_sanitised.clone())
                        .await
                    {
                        if verbose_mode() {
                            ctx.say(format!(
                                "Failed to ban {} because of {:?}",
                                member.display_name(),
                                error
                            ))
                            .await?;
                        } else {
                            ctx.say(format!("Failed to ban {}", member.display_name()))
                                .await?;
                        }
                        // 0u8
                    } else {
                        let phrase = phrase_list
                            .choose(&mut rand::thread_rng())
                            .expect("Unable to get meme phrase for ban");
                        ctx.say(format!("{} has been {phrase}", member.display_name()))
                            .await?;
                        // 0u8
                    };
                }
            }
        },
        None => {
            ctx.say("This must be ran from inside a guild").await?;
        }
    }
    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    guild_only,
    subcommands("ban_help", "ban_user")
)]
/// Ban a member or list of members by ID or Mention
pub async fn ban(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Run `/ban ban_help` to learn how to use this command and then use ")
        .await?;

    Ok(())
}

// TODO: Change to your own emojis!
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Admin", owners_only)]
/// Literally just shoot the bot!
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    let pewpew = ctx
        .say("<:GunPoint:908506214915276851> <:FBT:795660945627676712>")
        .await?;
    sleep(time::Duration::from_secs(1)).await;
    pewpew
        .edit(ctx, |b| {
            b.content("<:GunPoint:908506214915276851> 💥 <:FBT:795660945627676712>")
        })
        .await?;
    sleep(time::Duration::from_secs(1)).await;
    pewpew
        .edit(ctx, |b| {
            b.content("<:GunPoint:908506214915276851> <:FBT:795660945627676712>")
        })
        .await?;
    sleep(time::Duration::from_secs(1)).await;
    pewpew
        .edit(ctx, |b| {
            b.content("<:GunPoint:908506214915276851> 🩸 <:FBT:795660945627676712> 🩸")
        })
        .await?;
    sleep(time::Duration::from_secs(1)).await;
    ctx.say("Exiting now!").await?;

    #[cfg(feature = "database")]
    inc_execution_count().await?;

    let activity = Activity::playing("Sleeping");
    let status = OnlineStatus::Offline;

    ctx.serenity_context()
        .set_presence(Some(activity), status)
        .await;

    exit(0)
}

#[cfg(feature = "database")]
/// Authorize someone in this guild
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    check = "guild_auth_check",
    guild_only
)]
pub async fn authorize(
    ctx: Context<'_>,
    #[description = "User to authorise in this server"] user: Member,
) -> Result<(), Error> {
    ctx.defer_or_broadcast().await?;

    let uid = format!("{}", user.user.id.as_u64());

    let mut con = open_redis_connection().await?;

    // * json format: {users:[ID1, ID2, IDect]}
    let key_list: Option<HashSet<String>> = redis::cmd("SMEMBERS")
        .arg(format!(
            "authed-server-users:{}",
            ctx.guild_id().unwrap().as_u64()
        ))
        .clone()
        .query_async(&mut con)
        .await?;

    if let Some(list) = key_list {
        if list.contains(&uid) {
            ctx.say("User already authorised in this server!").await?;
        } else {
            match auth(ctx, &mut con, uid).await {
                Ok(()) => {
                    ctx.say(format!(
                        "{} is now authorized to use commands in this server!",
                        user.display_name()
                    ))
                    .await?;
                }
                Err(_error) if !verbose_mode() => {
                    ctx.say(format!("Failed to auth {}!", user.display_name()))
                        .await?;
                }
                Err(error) => {
                    ctx.say(format!(
                        "Failed to auth {}! Caused by {:?}",
                        user.display_name(),
                        error
                    ))
                    .await?;
                }
            }
        }
    } else {
        auth(ctx, &mut con, uid).await?;

        ctx.say(format!(
            "{} is now authorized to use commands in this server!",
            user.display_name()
        ))
        .await?;
    }

    Ok(())
}

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    check = "bot_admin_check"
)]
/// Send annoucement to any server that has been setup
pub async fn announcement(
    ctx: Context<'_>,
    #[description = "Title of announcement embed"] title: String,
    #[description = "Message to send to all servers (As a .txt file!)"] message_file: Attachment,
) -> Result<(), Error> {
    ctx.defer_or_broadcast().await?;

    let message_content = message_file.download().await?;
    let message = std::str::from_utf8(&message_content)?;

    let mut con = open_redis_connection().await?;

    let key_list: Vec<String> = redis::cmd("KEYS")
        .arg("guild-settings:*")
        .clone()
        .query_async(&mut con)
        .await?;

    let mut key_pipe = redis::pipe();

    for key in key_list {
        key_pipe.cmd("JSON.GET").arg(key);
    }

    let setting_entries: Vec<String> = key_pipe.atomic().query_async(&mut con).await?;

    let mut guild_settings_collection = Vec::new();
    for settings in setting_entries {
        let gs: GuildSettings = serde_json::from_str(&settings)?;

        guild_settings_collection.push(gs);
    }

    // TODO: Change to custom announcement message!

    let mut count: u64 = 0;
    for guild in guild_settings_collection.clone() {
        let colour = &mut rand::thread_rng().gen_range(0..10_000_000);
        match ChannelId(guild.channel_id.parse::<u64>()?).send_message(ctx, |f| {
            f.embed(|e| {
                e.title(format!("New announcement from FBT Security: {}", title.clone()))
                .description(message)
                .color(Colour::new(*colour))
                .author(|a| {
                    a.icon_url("https://cdn.discordapp.com/avatars/743269383438073856/959512463b1559b14818590d8c8a9d2a.webp?size=4096")
                    .name("FBT Security")
                })
                .thumbnail("https://media.giphy.com/media/U4sfHXAALLYBQzPcWk/giphy.gif")
            })
        }).await {
            Err(e)=>{
                event!(
                    Level::INFO,
                    "Failed to send announcement to a server because of" = ?e
                );
            },
            Ok(msg) => {
                count = count.add(1);
                println!("Sent to: {}", msg.link());
            },
        };
    }

    ctx.say(format!(
        "Sent annoucement to {}/{} servers!",
        count,
        guild_settings_collection.len()
    ))
    .await?;

    Ok(())
}

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
/// Request an FBT staff member to come and auth your server
pub async fn request_setup(
    ctx: Context<'_>,
    #[description = "Do you want to kick accounts that are under 90 days old"] alt_protection: bool,
) -> Result<(), Error> {
    ctx.defer().await?;
    let link = ChannelId(*ctx.channel_id().as_u64())
        .create_invite(ctx, |f| f.temporary(false).max_age(0).unique(false))
        .await?;


    // TODO: this channel is where the bot alterts you when a server is requesting use of the bot's moderation stuff
    ChannelId(953_435_498_318_286_898).send_message(ctx, |f| {
        f.content(format!("{0} is requesting authentication! {1}\n They requested for alt protection to be: `{alt_protection}`", ctx.guild().unwrap().name, link.url()))
    }).await?;

    ctx.send(|b| b.content("Request sent, sit tight!\nOnce an administrator joins make sure to give them permissions to acess the channel so they can set it up!").ephemeral(true)).await?;

    Ok(())
}

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    category = "Admin",
    check = "guild_auth_check",
    guild_only
)]
/// Setup your server's settings
pub async fn setup(
    ctx: Context<'_>,
    #[description = "Do you want to kick accounts that are under 90 days old when they join"]
    alt_protection: bool,
) -> Result<(), Error> {
    let mut con = open_redis_connection().await?;

    let guild_settings_json_in: Option<String> = redis::cmd("JSON.GET")
        .arg(format!(
            "guild-settings:{}",
            ctx.guild_id().unwrap().as_u64()
        ))
        .clone()
        .query_async(&mut con)
        .await?;

    let ch_id = format!("{}", ctx.channel_id().as_u64());
    let g_name = ctx
        .partial_guild()
        .await
        .expect("Unable to get Guild info")
        .name;

    if let Some(json_in) = guild_settings_json_in {
        let mut settings: GuildSettings = serde_json::from_str(&json_in)?;
        settings.channel_id = ch_id.clone();
        settings.kick = alt_protection;
        settings.server_name = g_name;

        set_guild_settings(ctx, &mut con, settings).await?;
        ctx.say(format!("Settings have been updated for your server!\nChannel for kick messages and bot announcements: <#{0}>.\nAlt protection: {alt_protection:?}.", ch_id.clone())).await?;
    } else {
        let settings = GuildSettings {
            channel_id: ch_id.clone(),
            kick: alt_protection,
            server_name: g_name,
        };

        set_guild_settings(ctx, &mut con, settings).await?;
        ctx.say(format!("Settings have been created for your server!\nChannel for kick messages and bot announcements: <#{0}>.\nAlt protection: {alt_protection:?}.", ch_id.clone())).await?;
    }

    Ok(())
}

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Admin",
    check = "guild_auth_check",
    guild_only
)]
/// Set your server's alt protection policy
pub async fn toggle_kick(ctx: Context<'_>) -> Result<(), Error> {
    let mut con = open_redis_connection().await?;

    let guild_settings_json_in: Option<String> = redis::cmd("JSON.GET")
        .arg(format!(
            "guild-settings:{}",
            ctx.guild_id().unwrap().as_u64()
        ))
        .clone()
        .query_async(&mut con)
        .await?;

    match guild_settings_json_in {
        // Update settings
        Some(json_in) => {
            let mut settings: GuildSettings = serde_json::from_str(&json_in)?;
            settings.kick = !settings.kick;

            set_guild_settings(ctx, &mut con, settings.clone()).await?;
            ctx.say(format!(
                "Settings have been updated for your server!\nAlt protection: {:?}.",
                settings.kick
            ))
            .await?;
        }
        // TODO: change to custom message
        // This should not be able to trigger because of the auth check but better safe than sorry
        None => {
            ctx.say("Your server has not been setup by a bot admin yet! Please context a bot admin or azuki to get authorised.").await?;
        }
    }

    Ok(())
}
