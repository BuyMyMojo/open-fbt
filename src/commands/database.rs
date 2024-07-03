use meilisearch_sdk::client::Client;
use merge::Merge;
use poise::serenity_prelude::Attachment;
use poise::serenity_prelude::{colours, AttachmentType, Colour, Member, UserId};
use rand::Rng;
use rusted_fbt_lib::checks::guild_auth_check;
use rusted_fbt_lib::structs::{BlacklistHit, CsvEntry, Offense, UserInfo};
use rusted_fbt_lib::utils::{open_redis_connection, verbose_mode};
use rusted_fbt_lib::vars::BlacklistOutput;
use rusted_fbt_lib::vars::{BOT_ADMINS, BOT_IDS, MEILISEARCH_API_KEY, MEILISEARCH_HOST};
use rusted_fbt_lib::{
    checks::bot_admin_check,
    types::{Context, Error},
};
use std::collections::HashSet;
use std::ops::{Add, Mul};
use tracing::instrument;
use tracing::{event, Level};

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "DB",
    member_cooldown = 1,
    required_permissions = "BAN_MEMBERS",
    check = "guild_auth_check",
    guild_only
)]
/// Add an ID to the bot
pub async fn add(
    ctx: Context<'_>,
    #[description = "Person to add to DB. Must be a user ID."] id: String,
    #[description = "Reason for being added to the DB."] reason: String,
    #[description = "Server ID this took place in. (Leave blank to use your own server ID)"]
    guild_id: Option<String>,
    #[description = "ID or URL to VRChat account."] vrc_id: Option<String>,
    #[description = "Link to image or google drive folder of images."] image: Option<String>,
    #[description = "Anything extra you want to add."] extra: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let mut is_not_user = false;

    let uid = id.trim().parse::<u64>().map_or_else(
        |_| {
            is_not_user = true;
            0
        },
        |i| i,
    );

    if is_not_user {
        ctx.send(|b| {
            b.content("Make sure you supplied a user ID")
                .ephemeral(true)
        })
        .await?;
    } else {
        let gid: String = guild_id.map_or_else(|| ctx.guild_id().unwrap().to_string(), |url| url);

        let mut con = open_redis_connection().await?;

        let result: Option<String> = redis::cmd("JSON.GET")
            .arg(format!("user:{uid}"))
            .clone()
            .query_async(&mut con)
            .await?;

        let uname = (UserId::from(uid).to_user(ctx).await)
            .map_or_else(|_| "NoUsernameFoundInDB".to_string(), |u| u.tag());

        match result {
            None => {
                let mut new: UserInfo = UserInfo {
                    vrc_id: None,
                    username: Some(uname),
                    discord_id: Some(format!("{uid}")),
                    offences: Vec::new(),
                };

                let mut new_offense = Offense {
                    guild_id: gid,
                    reason: reason.clone(),
                    image: image.clone().or_else(|| Some("N/A".to_string())),
                    extra: extra.clone().or_else(|| Some("N/A".to_string())),
                };

                match vrc_id {
                    None => {
                        new.vrc_id = Some("N/A".to_string());
                    }
                    Some(url) => {
                        new.vrc_id = Some(url);
                    }
                }

                match image {
                    None => {
                        new_offense.image = Some("N/A".to_string());
                    }
                    Some(url) => {
                        new_offense.image = Some(url);
                    }
                }

                match extra {
                    None => {
                        new_offense.extra = Some("N/A".to_string());
                    }
                    Some(url) => {
                        new_offense.extra = Some(url);
                    }
                }

                new.offences.push(new_offense);

                let json_user = serde_json::to_string(&new).unwrap();

                redis::cmd("JSON.SET")
                    .arg(format!("user:{uid}"))
                    .arg("$".to_string())
                    .arg(json_user)
                    .clone()
                    .query_async(&mut con)
                    .await?;
            }
            Some(_) => {
                let mut new_offense = Offense {
                    guild_id: gid,
                    reason: reason.clone(),
                    image: image.clone().or_else(|| Some("N/A".to_string())),
                    extra: extra.clone().or_else(|| Some("N/A".to_string())),
                };

                match image {
                    None => {
                        new_offense.image = Some("N/A".to_string());
                    }
                    Some(url) => {
                        new_offense.image = Some(url);
                    }
                }

                match extra {
                    None => {
                        new_offense.extra = Some("N/A".to_string());
                    }
                    Some(url) => {
                        new_offense.extra = Some(url);
                    }
                }

                let json_offense = serde_json::to_string(&new_offense).unwrap();

                redis::cmd("JSON.ARRAPPEND")
                    .arg(format!("user:{uid}"))
                    .arg("$.offences".to_string())
                    .arg(json_offense)
                    .clone()
                    .query_async(&mut con)
                    .await?;
            }
        }

        ctx.say(format!("<@{uid}> added into DB!\nReason: {reason}"))
            .await?;
    }

    Ok(())
}

// TODO: Change into sub command when we add vrc DB back
/// Search DB for yourself or with ID
#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "DB",
    member_cooldown = 5,
    check = "guild_auth_check",
    guild_only
)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Member to search for. This must be a user ID."] user_id: String,
) -> Result<(), Error> {
    use chrono::DateTime;
    use pastemyst::paste::*;
    use pastemyst::str;
    use poise::serenity_prelude::User;

    ctx.defer().await?;

    // let u = id.user.clone();

    let uid = user_id.trim().parse::<u64>()?;

    let u_opt: Option<User> = match UserId::from(uid).to_user(ctx).await {
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

    if let Some(u) = u_opt {
        let result = check_username_against_db(uid).await?;

        match result {
            None => {
                ctx.say(format!("No result found for {uid}.")).await?;
            }
            Some(hit) => {
                let hit_string: String = hit;
                let user: UserInfo = serde_json::from_str(hit_string.as_str())?;

                let username = user.username.unwrap_or_else(|| u.tag());
                let id = user.discord_id.unwrap_or(format!("{}", uid.clone()));
                let vrc = user.vrc_id.unwrap_or_else(|| "N/A".to_string());

                let field_count = user.offences.len() as u64;

                if field_count.mul(7).add(4).ge(&25_u64) {
                    let mut offences = String::new();

                    let mut i: u64 = 0;
                    for hit in user.offences.clone() {
                        i = i.add(1);

                        let image = hit.image.unwrap_or_else(|| "N/A".to_string());
                        let extra = hit.extra.unwrap_or_else(|| "N/A".to_string());
                        offences.push_str(format!("\n\nOffence {i}:\n Guild ID: {0}\n Reason: {1}\n Image(s): {image}\n Extra info: {extra}", hit.guild_id, hit.reason).as_str());
                    }

                    let msg_start = format!(
                        "Result found!\nUser has {} hit(s).\nUsername logged in DB: {}\nCurrent username: {}\nUser ID: {}\nVRChat ID: {}",
                        user.offences.clone().len() as u64,
                        username,
                        u.tag(),
                        id,
                        vrc
                        );

                    if (offences.chars().count() + msg_start.chars().count()) < 2000 {
                        // for format! merges msg_start and offences
                        ctx.say(format!("{msg_start}```{offences}```")).await?;
                    } else {
                        let txt_contents = format!("{msg_start}\n\n{offences}");
                        ctx.send(|f| {
                            f.content("Result found!\nThis user has so many hits that we can't display them all in Discord.\nBellow is a `.txt` file with all of their offences.")
                                .ephemeral(false)
                                .attachment(AttachmentType::Bytes {
                                    data: std::borrow::Cow::Borrowed(txt_contents.as_bytes()),
                                    filename: format!("{id}.txt"),
                                })
                        })
                        .await?;
                    }
                } else {
                    let mut fields: Vec<(String, String, bool)> = vec![
                        ("Username logged in DB:".to_string(), username.clone(), true),
                        ("Current username:".to_string(), u.tag(), true),
                        ("User ID:".to_string(), id.clone(), true),
                        ("VRChat ID:".to_string(), vrc.clone(), true),
                    ];

                    let mut i: u64 = 0;

                    // let mut buffer_len: usize = 0;

                    // for offense in user.offences.clone() {
                    //     buffer_len += offense.reason.chars().into_iter().count();
                    // }

                    for offense in user.offences.clone() {
                        i = i.add(1);
                        fields.push(("Offense:".to_string(), format!("#{i}"), false));

                        let image = offense.image.unwrap_or_else(|| "N/A".to_string());
                        let extra = offense.extra.unwrap_or_else(|| "N/A".to_string());

                        fields.push(("Guild ID:".to_string(), offense.guild_id, true));
                        fields.push((
                            "Reason:".to_string(),
                            {
                                if offense.reason.chars().into_iter().count() > 1000 {
                                    let pasties: Vec<PastyObject> = vec![PastyObject {
                                        _id: str!(""),
                                        language: str!(pastemyst::data::language::MARKDOWN),
                                        title: format!("{}'s long offence", username.clone()),
                                        code: offense.reason,
                                    }];

                                    let data: CreateObject = CreateObject {
                                        title: format!("{}'s long offence", username.clone()),
                                        expiresIn: String::from("1d"),
                                        isPrivate: false,
                                        isPublic: false,
                                        tags: String::from(""),
                                        pasties: pasties,
                                    };

                                    let paste = create_paste(data)?;

                                    format!("https://paste.myst.rs/{}", paste._id)
                                } else {
                                    offense.reason
                                }
                            },
                            true,
                        ));
                        fields.push(("Image(s):".to_string(), image, true));
                        fields.push(("Extra info:".to_string(), extra, true));
                    }

                    let colour = &mut rand::thread_rng().gen_range(0..10_000_000);

                    let msg = ctx.send(|b| {
                        b.embed(|e| {
                        e.title("Result found!")
                        .description(format!(
                        "User has {} hit(s).",
                        user.offences.clone().len() as u64
                        ))
                        .fields(fields)
                        .color(Colour::new(*colour))
                        .thumbnail(u.avatar_url().unwrap_or_else(|| "https://discord.com/assets/1f0bfc0865d324c2587920a7d80c609b.png".to_string()))
                        })
                        })
                        .await;
                    if msg.is_err() {
                        let msg_start = format!(
                            "Result found!\nUser has {} hit(s).\nUsername logged in DB: {}\nCurrent username: {}\nUser ID: {}\nVRChat ID: {}",
                            user.offences.clone().len() as u64,
                            username,
                            u.tag(),
                            id,
                            vrc
                            );

                        let mut offences = String::new();

                        let mut i: u64 = 0;
                        for hit in user.offences.clone() {
                            i = i.add(1);

                            let image = hit.image.unwrap_or_else(|| "N/A".to_string());
                            let extra = hit.extra.unwrap_or_else(|| "N/A".to_string());
                            offences.push_str(format!("\n\nOffence {i}:\n Guild ID: {0}\n Reason: {1}\n Image(s): {image}\n Extra info: {extra}", hit.guild_id, hit.reason).as_str());
                        }

                        let txt_contents = format!("{msg_start}\n\n{offences}");
                        ctx.send(|f| {
                                f.content("Result found!\nThere was some kinda of error sending the fancy version, most likely one of the field (for example the images) was too long.\nBellow is a `.txt` file with all of their offences formated as nicely as I could make it.")
                                    .ephemeral(false)
                                    .attachment(AttachmentType::Bytes {
                                        data: std::borrow::Cow::Borrowed(txt_contents.as_bytes()),
                                        filename: format!("{username}_{id}.txt"),
                                    })
                            })
                            .await?;
                    }
                }
            }
        }
    }

    let unix_timecode = rusted_fbt_lib::utils::snowflake_to_unix(u128::from(ctx.author().id.0));

    #[allow(clippy::cast_possible_truncation)]
    // this shouldn't be able to break but just in case I'm making the `unwrap_or` output NaiveDateTime::MIN
    let date_time_stamp = DateTime::from_timestamp(unix_timecode as i64, 0).unwrap_or(DateTime::UNIX_EPOCH);

    let age = chrono::Utc::now()
        .naive_utc()
        .signed_duration_since(date_time_stamp.naive_local())
        .num_days();

    let is_user_in_db: Option<String> = check_username_against_db(ctx.author().id.0).await.unwrap();


    // TODO: set your own channel ID!
    // log user name, id, guild name, id and url to channel
    poise::serenity_prelude::ChannelId(0000000000000000000)
        .send_message(ctx, |f| {
            f.embed(|e| {
                e.title("DB search info")
                    .field("Username", ctx.author().name.clone(), true)
                    .field("User ID", ctx.author().id.0.to_string(), true)
                    .field("User Account age (days)", age, true)
                    .field("Source Server Name", ctx.guild().unwrap().name, true)
                    .field(
                        "Source Server ID",
                        ctx.guild().unwrap().id.0.to_string(),
                        true,
                    )
                    .field("User ID Provided", user_id, true)
                    .field(
                        "Is user in DB (The person who ran command)",
                        format!("{}", is_user_in_db.is_some()),
                        false,
                    )
            })
        })
        .await?;

    Ok(())
}

pub async fn check_username_against_db(uid: u64) -> anyhow::Result<Option<String>> {
    let mut con = open_redis_connection().await?;
    let result: Option<String> = redis::cmd("JSON.GET")
        .arg(format!("user:{uid}"))
        .clone()
        .query_async(&mut con)
        .await?;
    Ok(result)
}

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "DB",
    member_cooldown = 5,
    check = "guild_auth_check",
    guild_only
)]
pub async fn remove_guild(
    ctx: Context<'_>,
    #[description = "ID of guild to remove from the DB."] guild_id: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let uid = guild_id.trim().to_string();

    let mut con = open_redis_connection().await?;

    // Create redis pipeline
    let mut pipe = redis::pipe();

    // Get all user keys
    let key_list: Vec<String> = redis::cmd("KEYS")
        .arg("user:*")
        .clone()
        .query_async(&mut con)
        .await?;

    // I wounder if rayon can be used here?
    for key in key_list {
        pipe.cmd("JSON.GET").arg(key);
    }

    // Should only fail if the DB is empty, if the DB is empty we have worse problems..
    let blacklist_entries_json: Vec<String> = pipe.atomic().query_async(&mut con).await?;

    // Vec os user info structs
    let mut blacklist_entries: Vec<UserInfo> = Vec::new();

    // Convert json to structs
    for entry in blacklist_entries_json {
        let entry: UserInfo = serde_json::from_str(entry.as_str())?;
        blacklist_entries.push(entry);
    }

    // Vec of user info structs that have offences in the guild we want to remove
    let mut guild_offences: Vec<UserInfo> = Vec::new();

    // Get all users that have offences in the guild we want to remove
    for entry in blacklist_entries {
        let entry_clone = entry.clone();
        for offence in entry.offences {
            if offence.guild_id.eq(&uid) {
                guild_offences.push(entry_clone);
                break;
            }
        }
    }

    // Create redis pipeline
    let mut pipe2 = redis::pipe();

    // Remove the guild from all users that have offences in it
    for entry in guild_offences.clone() {
        let mut offences: Vec<Offense> = Vec::new();

        for offence in entry.clone().offences {
            if !offence.guild_id.eq(&uid) {
                offences.push(offence);
            }
        }

        let mut user = entry.clone();
        user.offences = offences;

        let json_user = serde_json::to_string(&user)?;

        pipe2
            .cmd("JSON.SET")
            .arg(format!("user:{}", user.discord_id.unwrap()))
            .arg("$".to_string())
            .arg(json_user)
            .clone()
            .query_async(&mut con)
            .await?;
    }

    // Edit all entries at once
    pipe2.atomic().query_async(&mut con).await?;

    // Respond to the user with the amount of users that had offences in the guild we removed
    ctx.say(format!(
        "Removed guild {} from {} users.",
        guild_id,
        guild_offences.len()
    ))
    .await?;

    Ok(())
}

/// Update the search engine entries
#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(slash_command, category = "DB", owners_only)]
pub async fn update_search_engine(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Create a client (without sending any request so that can't fail)
    let client = Client::new(MEILISEARCH_HOST, Some(MEILISEARCH_API_KEY));

    // connect to index "entries"
    let entries = client?.index("entries");

    let msg = ctx
        .send(|b| b.content("Connected to search engine"))
        .await?;

    // connect to redis and pull all blacklist entries into a vec

    let mut con = open_redis_connection().await?;
    let mut pipe = redis::pipe();

    msg.edit(ctx, |b| b.content("Connected to DB")).await?;

    // Get all user keys
    let key_list: Vec<String> = redis::cmd("KEYS")
        .arg("user:*")
        .clone()
        .query_async(&mut con)
        .await?;

    msg.edit(ctx, |b| b.content("Fetching all DB entries"))
        .await?;

    // I wounder if rayon can be used here?
    for key in key_list {
        pipe.cmd("JSON.GET").arg(key);
    }

    // Should only fail if the DB is empty, if the DB is empty we have larger problems..
    let blacklist_entries: Vec<String> = pipe.atomic().query_async(&mut con).await?;

    let mut black_listed_users = Vec::new();
    for user in blacklist_entries {
        let user: UserInfo = serde_json::from_str(&user).expect("It was 701");

        black_listed_users.push(user);
    }

    msg.edit(ctx, |b| b.content("Updating search engine"))
        .await?;

    // push entries to meidisearch
    match entries
        .add_documents(black_listed_users.as_slice(), Some("discord_id"))
        .await
    {
        Ok(_) => {}
        Err(err) => {
            event!(
                Level::INFO,
                suggestion = "This error can probably be ignored?",
                error = ?err
            );
        }
    }

    msg.edit(ctx, |b| {
            b.content("All DB entries sent to search engine, any changes or new entries should be avaliable in a couple minutes.")
        }).await?;

    event!(Level::INFO, "Search engine was updated.");

    Ok(())
}

/// Check your entire server against the database. Now with output options!
#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "DB",
    guild_cooldown = 30,
    check = "guild_auth_check",
    guild_only
)]
pub async fn footprint_lookup(
    ctx: Context<'_>,
    #[description = "What format do you want the results as?"] output_format: Option<
        BlacklistOutput,
    >,
) -> Result<(), Error> {
    use chrono::DateTime;

    ctx.defer().await?;

    let mut con = open_redis_connection().await?;

    // let key_list: Vec<String> = redis::cmd("KEYS")
    //     .arg("user:*")
    //     .clone()
    //     .query_async(&mut con)
    //     .await?;

    let mut key_pipe = redis::pipe();

    let mut i = 1;
    let mut guild_members: Vec<Member> = Vec::new();

    while i < ctx.guild().unwrap().member_count {
        let latest = guild_members.last().map(|u| u.user.id);

        let mut guild_members_temp = ctx
            .guild()
            .unwrap()
            .members(ctx, Some(1000), latest)
            .await
            .expect("It was 710");

        i = i.add(1000);
        guild_members.append(&mut guild_members_temp);
    }

    // let mut blacklist_entries: Vec<String> = Vec::new();

    guild_members.clone().into_iter().for_each(|key| {
        key_pipe
            .cmd("JSON.GET")
            .arg(format!("user:{}", key.user.id.as_u64()));
    });

    let opt_blacklist_entries: Vec<Option<String>> =
        key_pipe.atomic().query_async(&mut con).await?;

    let mut blacklist_entries: Vec<String> = Vec::new();

    for ent in opt_blacklist_entries {
        let _ = ent.map_or((), |thing| {
            blacklist_entries.push(thing);
        });
    }

    let mut black_listed_users = Vec::new();
    for user in blacklist_entries {
        let user: UserInfo = serde_json::from_str(&user)?;

        black_listed_users.push(user);
    }

    let mut hit_count: u64 = 0;

    // Setup CSV
    let mut blacklist_file = csv::Writer::from_writer(vec![]);
    blacklist_file
        .write_record([
            "Discord ID",
            "Username",
            "Guild ID",
            "Reason",
            "Related image(s)",
            "Extra details",
        ])
        .expect("Unable to repare CSV");

    // Setup "Json"
    let mut struct_hit_list: Vec<BlacklistHit> = Vec::new();

    for member in guild_members {
        let member_id = format!("{}", member.user.id.as_u64());
        let hit_list: Vec<UserInfo> = black_listed_users
            .clone()
            .into_iter()
            .filter(|blu| {
                blu.discord_id
                    .as_ref()
                    .expect("Missing ID in BL user somehow?")
                    == &member_id
            })
            .collect();

        // Moves to next loop if empty
        if hit_list.is_empty() {
            continue;
        }

        hit_count += 1;

        // TODO: Add .json and other output options
        // TODO: Add compact output format:
        // @user1 - 1
        // @user2 - 5
        // @user3 - 2

        // Write discord specific instances to CSV
        for hit in hit_list {
            let cloned_hit = hit.clone();
            let username = UserId::from(cloned_hit.discord_id.as_ref().unwrap().parse::<u64>()?)
                .to_user(ctx)
                .await?
                .name
                .to_string();
            let user_id = cloned_hit
                .discord_id
                .expect("Missing ID in BL user somehow?")
                .to_string();

            match output_format {
                None | Some(BlacklistOutput::Csv) => {
                    for offense in hit.offences {
                        blacklist_file.write_record(&[
                            format!("'{}'", user_id.clone()),
                            format!("'{}'", username.clone()),
                            format!("'{}'", offense.guild_id),
                            format!("'{}'", offense.reason),
                            format!(
                                "'{}'",
                                offense
                                    .image
                                    .map_or_else(|| "N/A".to_string(), |image| image)
                            ),
                            format!(
                                "'{}'",
                                offense
                                    .extra
                                    .map_or_else(|| "N/A".to_string(), |extra| extra)
                            ),
                        ])?;
                    }
                }
                Some(
                    BlacklistOutput::Json | BlacklistOutput::Chat | BlacklistOutput::CompactChat,
                ) => {
                    for offense in hit.offences {
                        struct_hit_list.push(BlacklistHit {
                            user_id: user_id.clone(),
                            username: username.clone(),
                            guild_id: offense.guild_id.clone(),
                            reason: offense.reason.clone(),
                            image: (offense
                                .image
                                .map_or_else(|| "N/A".to_string(), |image| image))
                            .to_string(),
                            extra: (offense
                                .extra
                                .map_or_else(|| "N/A".to_string(), |extra| extra))
                            .to_string(),
                        });
                    }
                }
            }
        }
    }

    if hit_count > 0 {
        match output_format {
            None | Some(BlacklistOutput::Csv) => {
                // ? There has to be a btter way to handle this
                let blf_as_bytes = String::from_utf8(blacklist_file.into_inner().unwrap())
                    .expect("Unable to convert CSV data to string")
                    .as_bytes()
                    .to_owned();

                ctx.send(|b| {
                    b.content(format!(
                        "Your server has {} bad actor(s).\nA .csv file is attaches with all of the results, you can open this in any text editor but is more suited for google sheets/excel.\nrun `/ban ban_help` to find out how to ban multiple users at once.",
                        hit_count
                    ))
                    .attachment(AttachmentType::Bytes {
                        data: std::borrow::Cow::Borrowed(&blf_as_bytes),
                        filename: format!("{}_footprint_results.csv", 
                        match ctx.guild() {
                            Some(g) => g.name,
                            None => "your".to_string()
                        }),
                    })
                })
                .await?;
            }
            Some(BlacklistOutput::Json) => {
                let json_str = serde_json::to_string_pretty(&struct_hit_list)?;

                let blf_as_bytes = json_str.as_bytes().to_owned();

                ctx.send(|b| {
                    b.content(format!(
                        "Your server has {} bad actor(s).\nA .json file is attaches with all of the results.\nYou requested a json file so I'm going to assume you know what you're doing!",
                        hit_count
                    ))
                    .attachment(AttachmentType::Bytes {
                        data: std::borrow::Cow::Borrowed(&blf_as_bytes),
                        filename: format!("{}_footprint_results.json", 
                        match ctx.guild() {
                            Some(g) => g.name,
                            None => "your".to_string()
                        }),
                    })
                })
                .await?;
            }

            Some(BlacklistOutput::Chat) => {
                let mut message_content: Vec<String> = Vec::new();

                for hit in struct_hit_list {
                    let hit_msg = format!(
                        "<@{}>/{0} was found in your server for the following reason: `{}`\nExtras: {}\nImages: {}",
                        hit.user_id,
                        hit.reason,
                        hit.extra,
                        hit.image
                    );

                    let combined_message = message_content.join("\n-----\n");

                    // The +5 is to account for the `\n-----\n`
                    if (combined_message.chars().count() + hit_msg.chars().count() + 5) < 2000 {
                        message_content.push(hit_msg);
                    } else {
                        match ctx.say(combined_message.clone()).await {
                            Ok(_) => {}
                            Err(err) => {
                                event!(
                                    Level::ERROR,
                                    suggestion = "Message might be too large again?",
                                    error = ?err,
                                    msg_content = combined_message,
                                );
                                panic!();
                            }
                        }

                        message_content.clear();

                        if hit_msg.chars().count() > 2000 {
                            ctx.send(|f| {
                                f.content(format!(
                                    "<@{}>/{0} was found in your server for the following reason:",
                                    hit.user_id
                                ))
                                .ephemeral(false)
                                .attachment(
                                    AttachmentType::Bytes {
                                        data: std::borrow::Cow::Borrowed(hit_msg.as_bytes()),
                                        filename: format!(
                                            "{}-{}_extra_large_offence.txt",
                                            hit.user_id, hit.guild_id
                                        ),
                                    },
                                )
                            })
                            .await?;
                        } else {
                            message_content.push(hit_msg);
                        }
                    }
                }

                if !message_content.is_empty() {
                    ctx.say(message_content.join("\n-----\n")).await?;
                }

                ctx.say(format!("Found {hit_count} user(s) from the database!\nYou can easily ban them by right clicking on their @")).await?;
            }
            Some(BlacklistOutput::CompactChat) => {
                let mut message_content: Vec<String> = Vec::new();

                let mut unique_hits: HashSet<String> = HashSet::new();

                for hit in struct_hit_list {
                    unique_hits.insert(format!("Found: <@{0}>/{0}", hit.user_id));
                }

                for unique_hit in unique_hits {
                    if message_content.len() == 20 {
                        ctx.say(message_content.join("\n")).await?;
                        message_content.clear();
                    } else {
                        message_content.push(unique_hit);
                    }
                }

                if !message_content.is_empty() {
                    ctx.say(message_content.join("\n")).await?;
                }

                ctx.say(format!("Found {hit_count} user(s) from the database!\nYou can easily ban them by right clicking on their @")).await?;
            }
        }
    } else {
        ctx.say("Looks like the server is squeaky clean and free from cringe users, good job!")
            .await?;
    }

    let unix_timecode = rusted_fbt_lib::utils::snowflake_to_unix(u128::from(ctx.author().id.0));

    #[allow(clippy::cast_possible_truncation)]
    // this shouldn't be able to break but just in case I'm making the `unwrap_or` output NaiveDateTime::MIN
    let date_time_stamp = DateTime::from_timestamp(unix_timecode as i64, 0).unwrap_or(DateTime::UNIX_EPOCH);

    let age = chrono::Utc::now()
        .naive_utc()
        .signed_duration_since(date_time_stamp.naive_local())
        .num_days();

    let is_user_in_db: Option<String> =
        check_username_against_db(ctx.author().id.00).await.unwrap();


    // TODO: set your own channel ID!
    // log user name, id, guild name, id and url to channel
    poise::serenity_prelude::ChannelId(0000000000000000000)
        .send_message(ctx, |f| {
            f.embed(|e| {
                e.title("Entire server footprint request")
                    .field("Username", ctx.author().name.clone(), true)
                    .field("User ID", ctx.author().id.0.to_string(), true)
                    .field("User Account age (days)", age, true)
                    .field("Source Server Name", ctx.guild().unwrap().name, true)
                    .field(
                        "Source Server ID",
                        ctx.guild().unwrap().id.0.to_string(),
                        true,
                    )
                    .field(
                        "Is user in DB (The person who ran command)",
                        format!("{}", is_user_in_db.is_some()),
                        false,
                    )
            })
        })
        .await?;

    Ok(())
}

/// Process an entire CSV into the Redis database (V2.0)
#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    category = "DB",
    required_permissions = "ADMINISTRATOR",
    check = "bot_admin_check",
    guild_only
)]
pub async fn excel(
    ctx: Context<'_>,
    #[description = "CSV file to upload"] csv_file: Attachment,
    #[description = "ID of server the users are from"] guild_id: String,
    #[description = "Reason for being added to the DB"] reason: String,
) -> Result<(), Error> {
    ctx.defer_or_broadcast().await?;

    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    // Not actually case sensitive thanks to `.to_lowercase()`
    if csv_file.url.to_lowercase().ends_with(".csv") {
        let embed_message = ctx
            .send(|b| {
                b.embed(|e| {
                    e.description("Uploading entried to DB now")
                        .color(colours::css::WARNING)
                        .thumbnail("https://media1.giphy.com/media/3o7bu3XilJ5BOiSGic/giphy.gif")
                })
            })
            .await?;

        // Download file and convert to csv reader
        let response = reqwest::get(csv_file.url).await?;
        let csv_content = response.text().await?;
        let mut csv_reader = csv::Reader::from_reader(csv_content.as_bytes());

        // Start timer
        let start = tokio::time::Instant::now();

        let mut records = Vec::new();

        // Converts each line csv into a CsvEntry object
        for record in csv_reader.deserialize() {
            let entry: CsvEntry = match record {
                Err(err) => {
                    if verbose_mode() {
                        ctx.send(|b| {
                            b.embed(|e| {
                            e.description(format!("Failed to read a line of the CSV because: {err:?}"))
                            .color(colours::css::DANGER)
                            .thumbnail("https://media0.giphy.com/media/TqiwHbFBaZ4ti/giphy.gif?cid=ecf05e476gqwddci6tjtal5ohkp9ql3tq3m3scilolen1jh8&rid=giphy.gif&ct=g")
                            })
                            }).await?;
                    } else {
                        ctx.say("Failed to read a line of the CSV, probably a , in someone's name.\nYou can probably ignore this.".to_string())
                            .await?;
                    }
                    CsvEntry {
                        AuthorID: "0".to_string(),
                        Author: "0".to_string(),
                    }
                }
                Ok(out) => out,
            };

            if entry.AuthorID == *"0" && entry.Author == *"0" {
                continue;
            }

            records.push(entry);
        }

        #[allow(clippy::iter_with_drain)]
        let records_set: HashSet<_> = records.drain(..).collect(); // dedup
        records.extend(records_set.into_iter());

        // End conversion

        let mut con = open_redis_connection().await?;
        let mut pipe = redis::pipe();

        let mut users = Vec::new();
        let mut user_ids = Vec::new();

        for entry in records {
            // TODO: Add more checks once other lists are setup?
            if !BOT_IDS.contains(&entry.AuthorID.parse::<u64>()?)
                || BOT_ADMINS.contains(&entry.AuthorID.parse::<u64>()?)
            {
                let offense = vec![Offense {
                    guild_id: guild_id.parse().expect("Invalid Guild ID"),
                    reason: reason.clone(),
                    image: None,
                    extra: None,
                }];

                let user = UserInfo {
                    vrc_id: None,
                    username: Some(entry.Author),
                    discord_id: Some(entry.AuthorID),
                    offences: offense,
                };

                users.push(user.clone());
                user_ids.push(user.discord_id);
            }
        }

        #[allow(clippy::iter_with_drain)]
        let user_ids_set: HashSet<_> = user_ids.drain(..).collect(); // dedup
        user_ids.extend(user_ids_set.into_iter());

        let actual_count = user_ids.len() as u64;

        let mut combined_users = Vec::new();

        let key_list: Vec<String> = redis::cmd("KEYS")
            .arg("user:*")
            .clone()
            .query_async(&mut con)
            .await?;

        let mut key_pipe = redis::pipe();

        for key in key_list {
            key_pipe.cmd("JSON.GET").arg(key);
        }

        let old_users: Vec<String> = key_pipe.atomic().query_async(&mut con).await?;

        for old_user in old_users {
            let user: UserInfo = serde_json::from_str(&old_user)?;

            users.push(user);
        }

        for id in user_ids {
            let matches: Vec<UserInfo> = users
                .clone()
                .into_iter()
                .filter(|u| u.discord_id == id)
                .collect();

            let mut out_user = UserInfo {
                vrc_id: None,
                username: None,
                discord_id: None,
                offences: Vec::new(),
            };

            // Merge every match of this user into out_user
            for user in matches {
                out_user.merge(user);
            }
            let out_user_offences_set: HashSet<_> = out_user.offences.drain(..).collect(); // dedup
            out_user.offences.extend(out_user_offences_set.into_iter());

            combined_users.push(out_user);
        }

        for user in combined_users {
            let json_user = serde_json::to_string(&user).unwrap();

            // Queue up JSON.SET commands
            pipe.cmd("JSON.SET").arg(&[
                format!(
                    "user:{}",
                    user.discord_id.expect("Invalid Discord ID in entry")
                ),
                "$".to_string(),
                json_user,
            ]);
        }

        // Upload all entries at once
        pipe.atomic().query_async(&mut con).await?;

        // End timer
        let duration = start.elapsed();

        embed_message.edit(ctx, |b| {
        b.embed(|e| {
        e.description("Completed upload!".to_string())
        .color(colours::css::POSITIVE)
        .thumbnail("https://media3.giphy.com/media/mJHSkWKziszrkcNJPo/giphy.gif?cid=ecf05e47sli83f591onowkgacia9xezewha5pcoj6651yszz&rid=giphy.gif&ct=g")
        .fields([
        ("Upload time", format!("{duration:?}"), false),
        ("Entries processed", format!("{actual_count}"), false)
        ])
        })
        }).await?;
    } else {
        ctx.say("The file needs to be a `.csv`!").await?;
    }

    Ok(())
}

/// Print out the meaning behind C/R/L/T in blacklist entries
#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "DB")]
pub async fn key(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(
        "This is the Key for the Database: C/R/L/T\nC = Crasher, R = Ripper, L = Leaker, T = Toxic",
    )
    .await?;

    Ok(())
}

/// Add user id to alt protection whitelist
#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    check = "bot_admin_check",
    category = "DB"
)]
pub async fn whitelist(ctx: Context<'_>, user_id: String) -> Result<(), Error> {
    let mut con = open_redis_connection().await?;

    // get kick-whitelist as a hashset and check if user is already in it
    let kick_whitelist: HashSet<String> = redis::cmd("SMEMBERS")
        .arg("kick-whitelist")
        .clone()
        .query_async(&mut con)
        .await?;

    if kick_whitelist.contains(&user_id) {
        ctx.say("User is already in the whitelist!").await?;
    } else {
        // add user to kick-whitelist
        redis::cmd("SADD")
            .arg("kick-whitelist")
            .arg(user_id)
            .query_async(&mut con)
            .await?;

        ctx.say("User added to the whitelist!").await?;
    }

    Ok(())
}
