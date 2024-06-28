use chrono::NaiveDateTime;
use poise::serenity_prelude::{self as serenity, AttachmentType, RichInvite};
use rusted_fbt_lib::{
    checks::guild_auth_check,
    types::{Context, Error},
    utils::snowflake_to_unix,
};
use serde::Deserialize;
use tracing::instrument;

use crate::commands::database::check_username_against_db;

#[instrument(skip(ctx))]
#[poise::command(slash_command, track_edits, category = "Tools")]
/// Display your or another user's account creation date
pub async fn account_age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or_else(|| ctx.author());

    let uid = *user.id.as_u64();

    let unix_timecode = snowflake_to_unix(u128::from(uid));

    #[allow(clippy::cast_possible_truncation)]
    // this shouldn't be able to break but just in case I'm making the `unwrap_or` output NaiveDateTime::MIN
    let date_time_stamp =
        NaiveDateTime::from_timestamp_opt(unix_timecode as i64, 0).unwrap_or(NaiveDateTime::MIN);

    let age = chrono::Utc::now()
        .naive_utc()
        .signed_duration_since(date_time_stamp)
        .num_days();

    ctx.say(format!(
        "{}'s account was created at {}.\nSo They are {} days old.",
        user.name,
        user.created_at(),
        age
    ))
    .await?;

    Ok(())
}

/// Gets the creation date or a Snowflake ID
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Tools")]
pub async fn creation_date(
    ctx: Context<'_>,
    #[description = "ID of User/Message/Channel/ect"] snowflake_id: u128,
) -> Result<(), Error> {
    let unix_timecode = snowflake_to_unix(snowflake_id);

    #[allow(clippy::cast_possible_truncation)]
    // this shouldn't be able to break but just in case I'm making the `unwrap_or` output NaiveDateTime::MIN
    let date_time_stamp =
        NaiveDateTime::from_timestamp_opt(unix_timecode as i64, 0).unwrap_or(NaiveDateTime::MIN);

    ctx.say(format!("Created/Joined on {date_time_stamp}"))
        .await?;

    Ok(())
}

/// qmit
#[instrument(skip(ctx))]
#[poise::command(owners_only, slash_command, hide_in_help)]
pub async fn bot_owner_tool_1(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_list = ctx.serenity_context().cache.guilds();

    let mut invites: Vec<RichInvite> = Vec::new();

    for guild in guild_list {
        let guild_invites: Option<Vec<RichInvite>> = (guild.invites(ctx).await).ok();

        if guild_invites.clone().is_some() {
            invites.append(&mut guild_invites.unwrap());
        }
    }

    // let shit_list = format!("All invites the bot can see:\n\n{:?}", invites);

    let mut new_list: String = "Every invite the bot can see, grouped by guild:\n\n[\n".to_string();

    for invite in invites {
        new_list.push_str(format!("{},\n", serde_json::to_string(&invite)?).as_str());
    }

    new_list.push(']');

    ctx.send(|b| {
        b.content("All bot invites:".to_string())
            .attachment(AttachmentType::Bytes {
                data: std::borrow::Cow::Borrowed(new_list.as_bytes()),
                filename: format!("{}_invites.txt", ctx.id()),
            })
    })
    .await?;

    Ok(())
}

/// Get's all avaliable info from a Discord Invite
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "Tools",
    member_cooldown = 5,
    check = "guild_auth_check",
    guild_only
)]
pub async fn invite_info(
    ctx: Context<'_>,
    #[description = "Invite URL"] invite_url: String,
) -> Result<(), Error> {
    use linkify::LinkFinder;

    #[derive(Debug, Deserialize, Clone)]
    struct InviteObject {
        #[serde(rename(deserialize = "type"))]
        _type: u64,
        code: String,
        inviter: InviterObject,
        expires_at: Option<String>,
        guild: PartialGuild,
        guild_id: String,
        channel: PartialChannel,
        approximate_member_count: u64,
        approximate_presence_count: u64,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct InviterObject {
        id: String,
        username: String,
        #[allow(dead_code)]
        avatar: Option<String>,
        discriminator: Option<String>,
        #[allow(dead_code)]
        public_flags: u64,
        #[allow(dead_code)]
        flags: u64,
        #[allow(dead_code)]
        banner: Option<String>,
        #[allow(dead_code)]
        accent_color: Option<u64>,
        global_name: Option<String>,
        #[allow(dead_code)]
        avatar_decoration_data: Option<String>,
        #[allow(dead_code)]
        banner_color: Option<String>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct PartialGuild {
        #[allow(dead_code)]
        id: String,
        name: String,
        #[allow(dead_code)]
        splash: Option<String>,
        #[allow(dead_code)]
        banner: Option<String>,
        description: Option<String>,
        #[allow(dead_code)]
        icon: Option<String>,
        #[allow(dead_code)]
        features: Vec<String>,
        #[allow(dead_code)]
        verification_level: u64,
        vanity_url_code: Option<String>,
        #[allow(dead_code)]
        nsfw_level: u64,
        #[allow(dead_code)]
        nsfw: bool,
        premium_subscription_count: u64,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct PartialChannel {
        id: String,
        #[serde(rename(deserialize = "type"))]
        _type: u64,
        name: String,
    }

    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(&invite_url).collect();

    if links.is_empty() {
        ctx.say("No valid links found").await?;
        return Ok(());
    }

    let link_str = links[0].as_str().to_owned();

    let (_link, invite_code) = link_str.split_at(19);

    let response = reqwest::get(format!(
        "https://discord.com/api/v10/invites/{invite_code}?with_counts=true"
    ))
    .await?;
    let response_formatted: Option<InviteObject> = response.json().await?;

    if response_formatted.is_none() {
        ctx.say("Invite not found").await?;
        return Ok(());
    }

    let invite = response_formatted.unwrap();

    let invite_info_fields = vec![
        ("Code:", invite.code, false),
        ("Expires:", invite.expires_at.unwrap_or_default(), false),
        ("Destination channel name:", invite.channel.name, false),
        ("Destination channel ID:", invite.channel.id, false),
    ];

    let guild_info_fields = vec![
        ("Server name:", invite.guild.name, false),
        ("Server ID:", invite.guild_id, false),
        (
            "Server Description:",
            invite.guild.description.unwrap_or_default(),
            true,
        ),
        (
            "Vanity URL code:",
            invite.guild.vanity_url_code.unwrap_or_default(),
            false,
        ),
        (
            "Server boosts count:",
            format!("{}", invite.guild.premium_subscription_count),
            false,
        ),
        (
            "Approx member count:",
            format!("{}", invite.approximate_member_count),
            false,
        ),
        (
            "Approx online user count:",
            format!("{}", invite.approximate_presence_count),
            false,
        ),
    ];

    let inviter_info_fields = vec![
        ("Username:", invite.inviter.username, false),
        (
            "Global username:",
            invite.inviter.global_name.unwrap_or_default(),
            false,
        ),
        ("User ID:", invite.inviter.id.clone(), false),
        (
            "Discriminator(Eg: #0001):",
            invite.inviter.discriminator.unwrap_or_default(),
            false,
        ),
    ];

    ctx.send(|f| {
        f.embed(|e| e.title("Invite Info").fields(invite_info_fields))
            .embed(|e| e.title("Guild Info").fields(guild_info_fields))
            .embed(|e| e.title("Inviter Info").fields(inviter_info_fields))
    })
    .await?;

    let unix_timecode = snowflake_to_unix(u128::from(ctx.author().id.0));

    #[allow(clippy::cast_possible_truncation)]
    // this shouldn't be able to break but just in case I'm making the `unwrap_or` output NaiveDateTime::MIN
    let date_time_stamp =
        NaiveDateTime::from_timestamp_opt(unix_timecode as i64, 0).unwrap_or(NaiveDateTime::MIN);

    let age = chrono::Utc::now()
        .naive_utc()
        .signed_duration_since(date_time_stamp)
        .num_days();

    let is_user_in_db: Option<String> =
        check_username_against_db(invite.inviter.id.parse::<u64>().unwrap())
            .await
            .unwrap();


    // TODO: set your own channel ID!

    // log user name, id, guild name, id and url to channel
    serenity::ChannelId()
        .send_message(ctx, |f| {
            f.embed(|e| {
                e.title("User requested invite info")
                    .field("Username", ctx.author().name.clone(), true)
                    .field("User ID", ctx.author().id.0.to_string(), true)
                    .field("User Account age (days)", age, true)
                    .field("Source Server Name", ctx.guild().unwrap().name, true)
                    .field(
                        "Source Server ID",
                        ctx.guild().unwrap().id.0.to_string(),
                        true,
                    )
                    .field("Url provided", link_str, true)
                    .field(
                        "Is User in DB",
                        format!("{}", is_user_in_db.is_some()),
                        false,
                    )
            })
        })
        .await?;

    Ok(())
}
