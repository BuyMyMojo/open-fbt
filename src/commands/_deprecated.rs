// https://sqlitebrowser.org/
/// Transfer from sqlite DB, get the json files using "File>Export>Table(s) to json" in sqlitebrowser
#[cfg(feature = "database")]
#[allow(non_snake_case, non_camel_case_types)] // Keeping these badly names variables since that's what they are called in the SQLite DB
#[poise::command(slash_command, category = "Admin", owners_only, hide_in_help)]
async fn sqlite_transfer(
    ctx: Context<'_>,
    #[description = "vrc_data.json"] vrc_data: Attachment,
    // #[description = "no_bot_perms.json"] no_bot_perms: Attachment,
    #[description = "guild_channels.json"] guild_channels: Attachment,
    #[description = "authorized_users.json"] authorized_users: Attachment,
    #[description = "Cleared_IDs.json"] cleard_ids: Attachment,
    #[description = "Monitored Guilds.json"] monitored_guilds: Attachment,
) -> Result<(), Error> {
    ctx.defer().await?;

    #[derive(Debug, Deserialize)]
    struct AuthorizedUser {
        user_id: u64,
        guild_id: u64,
    }

    #[derive(Debug, Deserialize)]
    struct ClearedID {
        Discord_ID: u64,
        Name: String,
        Where_found: String,
        Cleared_Reason: String,
    }

    #[derive(Debug, Deserialize)]
    struct MonitoredGuild {
        Guild_Name: String,
        Guild_ID: u64,
        Invite_link: Option<String>,
        Updated: Option<String>,
        DMCA_Takedown_Nuked: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct GuildChannel {
        guild: u64,
        channel_id: u64,
        kick_active: u64,
        Name: String,
    }

    #[derive(Debug, Deserialize)]
    struct vrc_data {
        vrc_id: Option<String>,
        guild_id: Option<u64>,
        name: Option<String>,
        discord_id: u64,
        reason: String,
        image: Option<String>,
        extra: Option<String>,
    }

    let mut con = open_redis_connection().await?;
    let mut pipe = redis::pipe();

    let msg = ctx.say("Downloading files").await?;

    let authorized_users_json = authorized_users.download().await?;
    let guild_channel_json = guild_channels.download().await?;
    let cleard_ids_json = cleard_ids.download().await?;
    let monitored_guilds_json = monitored_guilds.download().await?;
    let vrc_data_json = vrc_data.download().await?;

    msg.edit(ctx, |b| b.content("Converting json to structs"))
        .await?;

    let auth_user_vec: Vec<AuthorizedUser> =
        serde_json::from_str(std::str::from_utf8(&authorized_users_json)?)?;
    let guild_channel_vec: Vec<GuildChannel> =
        serde_json::from_str(std::str::from_utf8(&guild_channel_json)?)?;
    let cleard_id_vec: Vec<ClearedID> =
        serde_json::from_str(std::str::from_utf8(&cleard_ids_json)?)?;
    let monitored_guilds_vec: Vec<MonitoredGuild> =
        serde_json::from_str(std::str::from_utf8(&monitored_guilds_json)?)?;
    let vrc_data_vec: Vec<vrc_data> = serde_json::from_str(std::str::from_utf8(&vrc_data_json)?)?;

    msg.edit(ctx, |b| b.content("Preparing authorized_users data"))
        .await?;

    for authed_user in auth_user_vec {
        pipe.cmd("SADD").arg(&[
            format!("authed-server-users:{}", authed_user.guild_id),
            format!("{}", authed_user.user_id),
        ]);
    }

    msg.edit(ctx, |b| b.content("Preparing guild_channels data"))
        .await?;

    for guild_settings in guild_channel_vec {
        let formatted = GuildSettings {
            channel_id: format!("{}", guild_settings.channel_id),
            kick: match guild_settings.kick_active {
                1 => true,
                0 => false,
                _ => false,
            },
            server_name: guild_settings.Name,
        };

        let json = serde_json::to_string(&formatted).unwrap();
        pipe.cmd("JSON.SET").arg(&[
            format!("guild-settings:{}", guild_settings.guild),
            "$".to_string(),
            json,
        ]);
    }

    msg.edit(ctx, |b| b.content("Preparing cleard_ids data"))
        .await?;

    for cleared_id in cleard_id_vec {
        let formatted: ClearedUser = ClearedUser {
            user_id: format!("{}", cleared_id.Discord_ID),
            username: cleared_id.Name,
            where_found: cleared_id.Where_found,
            reason: cleared_id.Cleared_Reason,
        };

        let json = serde_json::to_string(&formatted).unwrap();
        pipe.cmd("JSON.SET").arg(&[
            format!("cleared-user:{}", cleared_id.Discord_ID),
            "$".to_string(),
            json,
        ]);
    }

    msg.edit(ctx, |b| b.content("Preparing monitored_guilds data"))
        .await?;

    for guild in monitored_guilds_vec {
        let formatted: MonitoredGuildInfo = MonitoredGuildInfo {
            guild_name: guild.Guild_Name.to_string(),
            guild_id: format!("{}", guild.Guild_ID),
            invite_link: match guild.Invite_link {
                None => "N/A".to_string(),
                Some(link) => link.to_string(),
            },
            updated: match guild.Updated {
                None => "Never".to_string(),
                Some(date) => date.to_string(),
            },
            status: match guild.DMCA_Takedown_Nuked {
                None => "Unknown".to_string(),
                Some(status) => status.to_string(),
            },
        };

        let json = serde_json::to_string(&formatted).unwrap();
        pipe.cmd("JSON.SET").arg(&[
            format!("monitored-guild:{}", guild.Guild_ID),
            "$".to_string(),
            json,
        ]);
    }

    msg.edit(ctx, |b| b.content("Preparing vrc_data")).await?;

    let mut parsed_ids: HashSet<String> = HashSet::new();

    for user_data in vrc_data_vec {
        match parsed_ids.contains(&format!("{}", user_data.discord_id)) {
            false => {
                let mut new_user = UserInfo {
                    vrc_id: user_data.vrc_id,
                    username: user_data.name,
                    discord_id: Some(format!("{}", user_data.discord_id)),
                    offences: Vec::new(),
                };

                let offense = vec![Offense {
                    guild_id: match user_data.guild_id {
                        None => "N/A".to_string(),
                        Some(gid) => format!("{}", gid),
                    },
                    reason: user_data.reason,
                    image: user_data.image,
                    extra: user_data.extra,
                }];
                new_user.offences = offense;

                let json = serde_json::to_string(&new_user).unwrap();
                pipe.cmd("JSON.SET").arg(&[
                    format!("user:{}", user_data.discord_id),
                    "$".to_string(),
                    json,
                ]);

                parsed_ids.insert(format!("{}", user_data.discord_id));
            }
            true => {
                let offense = Offense {
                    guild_id: match user_data.guild_id {
                        None => "N/A".to_string(),
                        Some(gid) => format!("{}", gid),
                    },
                    reason: user_data.reason,
                    image: user_data.image,
                    extra: user_data.extra,
                };

                let json = serde_json::to_string(&offense).unwrap();
                pipe.cmd("JSON.ARRAPPEND")
                    .arg(format!("user:{}", user_data.discord_id))
                    .arg("$.offences".to_string())
                    .arg(json);
            }
        }
    }

    msg.edit(ctx, |b| b.content("Uploading data to DB")).await?;

    pipe.query_async(&mut con).await?;

    msg.edit(ctx, |b| b.content("All done!")).await?;

    Ok(())
}