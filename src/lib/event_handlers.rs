use crate::structs::{GuildSettings, UserInfo, WaybackResponse, WaybackStatus};
use crate::utils::snowflake_to_unix;
use crate::vars::FBT_GUILD_ID;
use chrono::NaiveDateTime;
use chrono::Utc;
use chrono_tz::Australia::Melbourne;
use colored::Colorize;
use poise::serenity_prelude::{self as serenity, ChannelId, Colour, MessageUpdateEvent};
use rand::Rng;
use std::collections::HashMap;
use tracing::{event, Level};

// TODO: Change to the ID of a channel you want all DMs sent to the bot to be relayed to
const DM_CHANNEL_ID: u64 = 0000000000000000000;

/// If enabled on a server it will warn them on black listed users joining
///
/// # Panics
///
/// Panics if unable to parse channel ID from DB to u64.
///
/// # Errors
///
/// This function will return an error if;
/// - Fails to contact redis DB.
/// - Fails to get guild settings from DB.
/// - Fails to ask Redis for coresponding DB entry for user.
/// - Fails to send message to channel.
#[cfg(feature = "database")]
pub async fn bl_warner(
    ctx: &serenity::Context,
    member: &serenity::Member,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::utils::open_redis_connection;

    let mut con = open_redis_connection().await?;

    let guild_settings_json_in: Option<String> = redis::cmd("JSON.GET")
        .arg(format!("guild-settings:{}", member.guild_id.as_u64()))
        .clone()
        .query_async(&mut con)
        .await?;

    let if_on_bl: Option<String> = redis::cmd("JSON.GET")
        .arg(format!("user:{}", member.user.id.as_u64()))
        .clone()
        .query_async(&mut con)
        .await?;

    match if_on_bl {
        None => {}
        Some(user_json) => {
            match guild_settings_json_in {
                None => {} // Do nothing
                // Check guild settings
                Some(server_json) => {
                    let settings: GuildSettings = serde_json::from_str(&server_json)?;
                    let user: UserInfo = serde_json::from_str(&user_json)?;

                    ChannelId::from(settings.channel_id.parse::<u64>().unwrap())
                        .say(
                            ctx,
                            format!(
                                "<@{}>/{0} Just joined your server with {} offenses on record",
                                user.discord_id.unwrap(),
                                user.offences.len()
                            ),
                        )
                        .await?;
                }
            }
        }
    }

    Ok(())
}

/// Checks if server has alt protection enabled and then kicks the new member if they are >90 days old
///
/// # Errors
///
/// This function will return an error if;
/// - Fails to connect to Redis DB.
/// - Fails to serde guild settings json to `GuildSettings` struct.
/// - Fails to send DM to user getting kicked.
/// - Fails to actually kick member.
///
#[cfg(feature = "database")]
pub async fn alt_kicker(
    ctx: &serenity::Context,
    member: &serenity::Member,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::utils::open_redis_connection;
    use std::collections::HashSet;

    let mut con = open_redis_connection().await?;

    let whitelist: HashSet<String> = redis::cmd("SMEMBERS")
        .arg("kick-whitelist")
        .clone()
        .query_async(&mut con)
        .await?;

    if whitelist.contains(&member.user.id.0.to_string()) {
        return Ok(()); // Don't kick whitelisted users
    }

    let guild_settings_json_in: Option<String> = redis::cmd("JSON.GET")
        .arg(format!("guild-settings:{}", member.guild_id.as_u64()))
        .clone()
        .query_async(&mut con)
        .await?;

    match guild_settings_json_in {
        None => {} // Do nothing
        // Check guild settings
        Some(json_in) => {
            let settings: GuildSettings = serde_json::from_str(&json_in)?;
            // Is kicking enabled?
            if settings.kick {
                let uid = *member.user.id.as_u64();

                // Trying to handle the pfp here to see if it catches more or maybe most alts really do have the same pfp
                let pfp = member
                    .avatar_url()
                    .unwrap_or_else(|| {
                        "https://discord.com/assets/1f0bfc0865d324c2587920a7d80c609b.png"
                            .to_string()
                    })
                    .clone();

                let unix_timecode = snowflake_to_unix(u128::from(uid));

                #[allow(clippy::pedantic)]
                // it literally only take's i64, no need to warn about truncation here.
                let date_time_stamp = NaiveDateTime::from_timestamp_opt(unix_timecode as i64, 0)
                    .unwrap_or(NaiveDateTime::MIN);

                let age = chrono::Utc::now()
                    .naive_utc()
                    .signed_duration_since(date_time_stamp)
                    .num_days();

                // Compare user age
                if !age.ge(&90_i64) {
                    member.user.direct_message(ctx.http.clone(), |f| {
                        f.content("It looks like your account is under 90 days old, or has been detected as a potential alt. You have been kick from the server!\nYou have not been banned, feel free to join back when your account is over 90 days old.\nRun the `about` slash command or send `help in this chat to find out more.")
                    }).await?;
                    member
                        .kick_with_reason(
                            ctx.http.clone(),
                            &format!("Potential alt detected, account was {age:.0} day(s) old"),
                        )
                        .await?;

                    let colour = &mut rand::thread_rng().gen_range(0..10_000_000);

                    ChannelId(settings.channel_id.parse::<u64>()?)
                        .send_message(ctx.http.clone(), |f| {
                            f.embed(|e| {
                                e.title("Alt kicked!")
                                    .description(format!(
                                        "Potential alt detected, account was {:.0} day(s) old",
                                        age
                                    ))
                                    .thumbnail(pfp)
                                    .field("User ID", uid, true)
                                    .field("Name", member.user.name.clone(), true)
                                    .color(Colour::new(*colour))
                            })
                        })
                        .await?;
                }
            }
        }
    }
    Ok(())
}

/// Sends all recieved DMs into a specified channel
///
/// # Errors
///
/// This function will return an error if;
/// - Fails to handle message attachments.
/// - Fails to handle message stickers.
/// - Fails to send request to wayback machine.
/// - Fails to send message to DM channel.
// TODO: Handle attachments, list of links?
pub async fn handle_dms(
    event: &serenity::Message,
    ctx: &serenity::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !event.author.bot {
        let message = event.clone();
        let uid = *message.author.id.as_u64();

        let icon = message.author.avatar_url().map_or_else(
            || "https://discord.com/assets/1f0bfc0865d324c2587920a7d80c609b.png".to_string(),
            |url| url,
        );

        let cache = ctx.http.clone();

        let colour = &mut rand::thread_rng().gen_range(0..10_000_000);

        let now = Utc::now().with_timezone(&Melbourne);

        let local_time = now.to_string();

        let timestamp = local_time.to_string();

        let mut wayback_job_ids = Vec::new();

        let list_of_files = if message.attachments.is_empty() | message.sticker_items.is_empty() {
            "N/A".to_string()
        } else {
            let mut urls = Vec::new();

            handle_files(&message, &mut wayback_job_ids, &mut urls).await?;

            // Duped code for stickers, could probably refactor into function
            handle_stickers(&message, ctx, &mut wayback_job_ids, &mut urls).await?;

            urls.join("\n \n")
        };

        let mut msg = ChannelId(DM_CHANNEL_ID)
            .send_message(cache, |f| {
                f.embed(|e| {
                    e.title("New message:")
                        .description(message.content.clone())
                        .field("Attachments/Stickers:", list_of_files.clone(), false)
                        .field("User ID", uid, false)
                        .field("Recieved at:", timestamp.clone(), false)
                        .author(|a| a.icon_url(icon.clone()).name(message.author.name.clone()))
                        .color(Colour::new(*colour))
                })
            })
            .await?;

        let mut wayback_urls: Vec<String> = Vec::new();

        for job in wayback_job_ids {
            let mut is_not_done = true;
            while is_not_done {
                let client = reqwest::Client::new();

                // TODO: Change to your own wayback machine authorization key
                let response = client
                    .get(format!("https://web.archive.org/save/status/{job}"))
                    .header("Accept", "application/json")
                    .header("Authorization", "LOW asdgasdg:fasfaf") // auth key here!!
                    .send()
                    .await?;
                let response_content = response.text().await?;
                let wayback_status: WaybackStatus = serde_json::from_str(&response_content)?;

                if wayback_status.status == *"success" {
                    wayback_urls.push(format!(
                        "https://web.archive.org/web/{}",
                        wayback_status.original_url.unwrap_or_else(|| {
                            "20220901093722/https://www.dafk.net/what/".to_string()
                        })
                    ));
                    is_not_done = false;
                }
            }
        }

        if !wayback_urls.is_empty() {
            msg.edit(ctx, |f| {
                f.embed(|e| {
                    e.title("New message:")
                        .description(message.content.clone())
                        .field("Attachments/Stickers:", list_of_files, false)
                        .field(
                            "Archived Attachments/Stickers:",
                            wayback_urls.join("\n \n"),
                            false,
                        )
                        .field("User ID", uid, false)
                        .field("Recieved at:", timestamp, false)
                        .author(|a| a.icon_url(icon).name(message.author.name.clone()))
                        .color(Colour::new(*colour))
                })
            })
            .await?;
        }
    }

    Ok(())
}

/// Handles DM files.
///
/// # Errors
///
/// This function will return an error if Failes to contact wayback machine.
async fn handle_files(
    message: &serenity::Message,
    wayback_job_ids: &mut Vec<String>,
    urls: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for file in message.attachments.clone() {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("url".to_string(), file.url.clone());
        params.insert("skip_first_archive".to_string(), "1".to_string());

        // TODO: Change to your own wayback machine authorization key

        let response = client
            .post("https://web.archive.org/save")
            .form(&params)
            .header("Accept", "application/json")
            .header("Authorization", "LOW asdgasdg:fasfaf")
            .send()
            .await?;

        let response_content = response.text().await?;
        let wayback_status: WaybackResponse = serde_json::from_str(&response_content)?;

        if wayback_status.status.is_none() {
            if let Some(jid) = wayback_status.job_id {
                wayback_job_ids.push(jid);
            }
        }

        urls.push(file.url);
    }
    Ok(())
}

/// Handles DM stickers.
///
/// # Errors
///
/// This function will return an error if Failes to contact wayback machine.
async fn handle_stickers(
    message: &serenity::Message,
    ctx: &serenity::Context,
    wayback_job_ids: &mut Vec<String>,
    urls: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for file in message.sticker_items.clone() {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert(
            "url".to_string(),
            file.to_sticker(ctx)
                .await
                .unwrap()
                .image_url()
                .unwrap()
                .clone(),
        );
        params.insert("skip_first_archive".to_string(), "1".to_string());

        // TODO: Change to your own wayback machine authorization key

        let response = client
            .post("https://web.archive.org/save")
            .form(&params)
            .header("Accept", "application/json")
            .header("Authorization", "LOW asdgasdg:fasfaf")
            .send()
            .await?;

        let response_content = response.text().await?;
        let wayback_status: WaybackResponse = serde_json::from_str(&response_content)?;

        match wayback_status.status {
            None => {
                if let Some(jid) = wayback_status.job_id {
                    wayback_job_ids.push(jid);
                }
            }
            Some(_) => {}
        }

        urls.push(file.to_sticker(ctx).await.unwrap().image_url().unwrap());
    }
    Ok(())
}

/// When a message is edited in FBT this function will send the new and old message to a specified channel.
///
/// # Panics
///
/// Panics if an author doesn't exist, should be unreachable.
///
/// # Errors
///
/// This function will return an error if the message fails to send.
pub async fn handle_msg_edit(
    event: MessageUpdateEvent,
    old_if_available: &Option<serenity::Message>,
    ctx: &serenity::Context,
    new: &Option<serenity::Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if event.guild_id.is_some() {
        if let Some(author) = event.author.clone() {
            if !author.bot {
                let old_message = old_if_available.as_ref().map_or_else(
                    || "Message not stored in cache :(".to_string(),
                    |msg| msg.content.to_string(),
                );

                let new_message = new.as_ref().map_or_else(
                    || "Message not stored in cache :(".to_string(),
                    |msg| msg.content.to_string(),
                );

                let message_url = new.as_ref().map_or_else(
                    || "URL stored in cache :(".to_string(),
                    poise::serenity_prelude::Message::link,
                );

                let current_time = Utc::now().with_timezone(&Melbourne);

                let local_time = current_time.to_string();

                let timestamp = local_time.to_string();

                // TODO: channel to alert you that a message has been deleted

                ChannelId(891_294_507_923_025_951)
                    .send_message(ctx.http.clone(), |f| {
                        f.embed(|e| {
                            e.title(format!(
                                "\"{}\" Edited a message",
                                event.author.clone().unwrap().tag()
                            ))
                            .field("Old message content:", old_message, false)
                            .field("New message content:", new_message, false)
                            .field("Link:", message_url, false)
                            .field("Edited at:", timestamp, false)
                            .footer(|f| {
                                f.text(format!(
                                    "User ID: {}",
                                    event.author.clone().unwrap().id.as_u64()
                                ))
                            })
                            .color(Colour::new(0x00FA_A81A))
                        })
                    })
                    .await?;
            }
        }
    };
    Ok(())
}

/// Handles messages that have been deleted
///
/// # Panics
///
/// Panics if there is no message object in cache.
///
/// # Errors
///
/// This function will return an error if unable to send message to channel.
pub async fn handle_msg_delete(
    guild_id: &Option<serenity::GuildId>,
    ctx: &serenity::Context,
    channel_id: &ChannelId,
    deleted_message_id: &serenity::MessageId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match guild_id {
        None => {}
        Some(gid) => {
            // TODO: this logs any delted message in FBT specifically, change to your own server ID
            if *gid.as_u64() == 737_168_134_502_350_849 {
                match ctx.cache.message(channel_id, deleted_message_id) {
                    None => {}
                    Some(msg) => {
                        if !msg.author.bot {
                            let message = match ctx.cache.message(channel_id, deleted_message_id) {
                                None => "Message not stored in cache :(".to_string(),
                                Some(msg) => format!("{:?}", msg.content),
                            };

                            let author_id = match message.as_str() {
                                "Message not stored in cache :(" => 0_u64,
                                _ => *ctx
                                    .cache
                                    .message(channel_id, deleted_message_id)
                                    .unwrap()
                                    .author
                                    .id
                                    .as_u64(),
                            };

                            let author_tag =
                                if message.clone().as_str() == "Message not stored in cache :(" {
                                    "Not in cache#000".to_string()
                                } else {
                                    format!(
                                        "{:?}",
                                        match ctx.cache.message(channel_id, deleted_message_id) {
                                            Some(msg) => {
                                                msg.author.tag()
                                            }
                                            None => {
                                                String::new() // This just creates ""
                                            }
                                        }
                                    )
                                };

                            let now = Utc::now().with_timezone(&Melbourne);

                            let local_time = now.to_string();

                            let timestamp = local_time.to_string();

                            // TODO: This is the channel the deleted messages are sent to

                            ChannelId(891_294_507_923_025_951)
                                .send_message(ctx.http.clone(), |f| {
                                    f.embed(|e| {
                                        e.title(format!("{author_tag} deleted a message"))
                                            .field("Message content:", message, false)
                                            .field("Deleted at:", timestamp, false)
                                            .field(
                                                "Channel link:",
                                                format!(
                                                    "https://discord.com/channels/{}/{}",
                                                    guild_id
                                                        .unwrap_or(serenity::GuildId::from(
                                                            FBT_GUILD_ID
                                                        ))
                                                        .as_u64(),
                                                    channel_id.as_u64()
                                                ),
                                                false,
                                            )
                                            .footer(|f| f.text(format!("User ID: {author_id}")))
                                            .color(Colour::new(0x00ED_4245))
                                    })
                                })
                                .await?;
                        }
                    }
                }
            }
        }
    };
    Ok(())
}

/// Prints message and outputs trace if in verbose mode
pub fn handle_resume(event: &serenity::ResumedEvent) {
    event!(
        Level::INFO,
        "ResumedEvent" = format!(
            "{}",
            "Bot went offline but is online again".bright_red().italic()
        )
    );

    // Is this a good idea?
    event!(
        Level::TRACE,
        "ResumedEvent" = format!(
            "{}",
            "Bot went offline but is online again".bright_red().italic()
        ),
        "event" = ?event
    );
}
