use poise::serenity_prelude::{self as serenity};
use poise::serenity_prelude::{ChannelId, RoleId};
use poise::serenity_prelude::{PermissionOverwrite, PermissionOverwriteType, Permissions};
use rusted_fbt_lib::enums::CloseTicketFail;
use rusted_fbt_lib::types::{Context, Error};
use rusted_fbt_lib::utils::verbose_mode;
use rusted_fbt_lib::vars::{CLOSED_TICKET_CATEGORY, FBT_GUILD_ID, TICKET_CATEGORY};
use tracing::instrument;

#[instrument(skip(ctx))]
#[poise::command(slash_command, category = "Ticket", guild_only)]
/// Create new ticket (FBT discord only!)
pub async fn new_ticket(
    ctx: Context<'_>,
    #[description = "An optional topic to put on the ticket"] topic: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if *ctx.guild_id().unwrap().as_u64() == FBT_GUILD_ID {
        let mut channels = Vec::new();

        for channel in ctx.guild().unwrap().channels {
            let parent_id = match channel.1.clone() {
                serenity::Channel::Guild(g) => g.parent_id,
                _ => None,
            };

            if let Some(cat) = parent_id {
                if *cat.as_u64() == TICKET_CATEGORY {
                    channels.push(channel.0.name(ctx).await.unwrap());
                }
            }
        }

        let mut existing_ticket = false;

        for ch in channels {
            if ch.starts_with(&ctx.author().name) {
                existing_ticket = true;
            }
        }

        if existing_ticket {
            ctx.send(|b| {
                b.content("You already have a ticket open, you cannot open another!")
                    .ephemeral(true)
            })
            .await?;
        } else {

            // TODO: change these IDs to be your own server roles

            // Assuming a guild has already been bound.
            let perms = vec![
                PermissionOverwrite {
                    allow: Permissions::READ_MESSAGE_HISTORY
                        | Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::ADD_REACTIONS
                        | Permissions::EMBED_LINKS
                        | Permissions::ATTACH_FILES
                        | Permissions::USE_EXTERNAL_EMOJIS,
                    deny: Permissions::empty(),
                    kind: PermissionOverwriteType::Member(ctx.author().id),
                },
                PermissionOverwrite {
                    allow: Permissions::all(),
                    deny: Permissions::SEND_TTS_MESSAGES,
                    kind: PermissionOverwriteType::Role(RoleId::from(737_168_134_569_590_888)), // Secretary (Probably not needed)
                },
                PermissionOverwrite {
                    allow: Permissions::all(),
                    deny: Permissions::SEND_TTS_MESSAGES,
                    kind: PermissionOverwriteType::Role(RoleId::from(820_914_502_220_513_330)), // Admin (Probably not needed)
                },
                PermissionOverwrite {
                    allow: Permissions::all(),
                    deny: Permissions::SEND_TTS_MESSAGES,
                    kind: PermissionOverwriteType::Role(RoleId::from(874_898_210_534_096_907)), // Mods
                },
                PermissionOverwrite {
                    allow: Permissions::all(),
                    deny: Permissions::SEND_TTS_MESSAGES,
                    kind: PermissionOverwriteType::Role(RoleId::from(1_005_994_060_416_294_942)), // World admin panel
                },
                PermissionOverwrite {
                    allow: Permissions::all(),
                    deny: Permissions::SEND_TTS_MESSAGES,
                    kind: PermissionOverwriteType::Role(RoleId::from(1_046_937_023_400_919_091)), // World admin panel trainee
                },
                PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::all(),
                    kind: PermissionOverwriteType::Role(RoleId::from(737_168_134_502_350_849)), // @everyone
                },
            ];

            match ctx
                .guild()
                .expect("")
                .create_channel(ctx, |c| {
                    c.category(ChannelId::from(TICKET_CATEGORY))
                        .name(format!(
                            "{}-{}",
                            ctx.author().name,
                            chrono::offset::Utc::now().format("%s")
                        ))
                        .permissions(perms)
                        .topic(topic.unwrap_or_else(|| "A new ticket".to_string()))
                })
                .await
            {
                Ok(ch) => {
                    ctx.send(|b| {
                        b.content(format!(
                            "Ticket created! Find it here: <#{}>",
                            ch.id.as_u64()
                        ))
                        .ephemeral(true)
                    })
                    .await?;

                    ch.say(
                        ctx,
                        format!("New ticket opened by <@{}>!", ctx.author().id.as_u64()),
                    )
                    .await?;
                }
                Err(error) => {
                    let err_msg = if verbose_mode() {
                        format!("Failed to create ticket. Reason: {error:?}")
                    } else {
                        "Failed to create ticket".to_string()
                    };

                    ctx.send(|b| b.content(err_msg).ephemeral(true)).await?;
                }
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, category = "Ticket", guild_only)]
/// Closes the current ticket (FBT discord only!)
pub async fn close_ticket(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if *ctx.guild_id().unwrap().as_u64() == FBT_GUILD_ID {
        let mut failed = CloseTicketFail::False;

        let current_channel = ctx.channel_id().to_channel(ctx).await?;
        let chnnl_name = ctx
            .channel_id()
            .name(ctx)
            .await
            .unwrap_or_else(|| "Unkown Ticket".to_string());

        let parent_id = match current_channel {
            serenity::Channel::Guild(g) => g.parent_id,
            _ => None,
        };

        match parent_id {
            None => {
                failed = CloseTicketFail::False;
            }
            Some(channel_category) => {
                if *channel_category.as_u64() == TICKET_CATEGORY {
                    match ctx
                        .channel_id()
                        .edit(ctx, |c| {
                            c.category(Some(ChannelId::from(CLOSED_TICKET_CATEGORY)))
                                .name(format!(
                                    "{}-{}",
                                    chnnl_name,
                                    chrono::offset::Utc::now().format("%s")
                                ))
                        })
                        .await
                    {
                        Ok(_) => {}
                        Err(fail_reason) => {
                            failed = CloseTicketFail::SerenityError(fail_reason);
                        }
                    }
                } else {
                    failed = CloseTicketFail::IncorrectCategory;
                }
            }
        }

        match failed {
            CloseTicketFail::False => {
                ctx.say("Ticket closed!").await?;
            }
            CloseTicketFail::IncorrectCategory => {
                ctx.send(|b| {
                    b.content(format!(
                        "This can only be ran inside of a channel under <#{}>!",
                        TICKET_CATEGORY
                    ))
                    .ephemeral(true)
                })
                .await?;
            }
            CloseTicketFail::SerenityError(error) => {
                ctx.send(|b| {
                    b.content(format!(
                        "Failed to close ticker because of following error:\n{}",
                        error
                    ))
                    .ephemeral(true)
                })
                .await?;
            }
        }
    } else {
        ctx.say("This command must be ran inside of FBT's discord")
            .await?;
    }

    Ok(())
}
