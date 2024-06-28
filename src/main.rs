#![forbid(unsafe_code)]

//! Please increade the version in the Cargo.toml file by 0.0.1 for &&every minor commit or command and by 0.1.0 for any majoy function rewrite or implamentation

// TODO: Add ticket ssytem
// ? /close_ticket could check a DB list to see if it contains the channel ID and if it does then close?
// ? If we wan't more info we can store each ticket as a json file and then only close if an entry for the channel exists in DB

use clap::Parser;
use colored::Colorize;
use commands::database::remove_guild;
use poise::builtins::register_application_commands_buttons;
use poise::serenity_prelude::{self as serenity, ChannelId, Colour, UserId};
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use std::collections::HashSet;
use std::fs::File;
use tracing::instrument;
use tracing::metadata::LevelFilter;
use tracing::{event, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[macro_use]
extern crate maplit;

#[cfg(feature = "database")]
use rusted_fbt_lib::utils::open_redis_connection;
use rusted_fbt_lib::vars::FBT_GUILD_ID;
// Import everything from the commands folder
mod commands;
use commands::admin::{
    announcement, authorize, ban, botmsg, request_setup, setup, shutdown, toggle_kick,
};
#[cfg(feature = "database")]
use commands::database::{
    add, excel, footprint_lookup, key, search, update_search_engine, whitelist,
};
use commands::fun::{cringe, owo, ping, pog, toxic, waifu};
use commands::info::{about, feedback, help};
use commands::tickets::{close_ticket, new_ticket};
use commands::tools::{account_age, bot_owner_tool_1, creation_date};

// New rust librabry to never leave this reposity :D
use rusted_fbt_lib::args::Args;
use rusted_fbt_lib::checks::bot_admin_check;
use rusted_fbt_lib::enums::{DebugLevel, LogDebugLevel};
use rusted_fbt_lib::event_handlers::{
    alt_kicker, bl_warner, handle_dms, handle_msg_delete, handle_msg_edit, handle_resume,
};
use rusted_fbt_lib::memes::pog_be_gone;
use rusted_fbt_lib::structs::{Data, PasteResponse};
use rusted_fbt_lib::types::{Context, Error};
use rusted_fbt_lib::utils::inc_execution_count;

use crate::commands::tools::invite_info;

/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, hide_in_help, owners_only)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    register_application_commands_buttons(ctx).await?;
    event!(Level::INFO, "Commandwhere registered");
    Ok(())
}

/// Custom error handeling
#[instrument]
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        // allow unused_variables because we don't use all the variables
        #[allow(unused_variables)]
        poise::FrameworkError::Setup {
            error,
            framework,
            data_about_bot,
            ctx,
        } => {
            // Log failed bot setup
            event!(Level::ERROR, "Bot setup failed" = ?error);
        }
        poise::FrameworkError::Command { error, ctx } => {
            event!(Level::WARN, "Error in command" = ?ctx.command(), "error" = ?error);
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx } => {
            if ctx.command().name.as_str() == "setup" {
                ctx.send(|m| {
                        m.content("If you can't run this because you don't have the correct permissions then please ask a local admin to run `/request_setup` or `/authorize`!\nAn admin will come and check out your server ASAP after `/request_setup` is executed.")
                        .ephemeral(true)
                    }).await
                    .expect("Failed to tell user about request_setup during error handeling");
            }

            event!(Level::INFO, "CommandCheckFailed" = ?ctx.command(), "error" = ?error);
        }
        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => {
            ctx.say(format!("I'm currently missing the follow permission(s) required to execute this command:\n\n```{}```\n\nPlease ask a local server admin to fix this in my bot role!", missing_permissions.get_permission_names().join("\n"))).await
            .expect("Unable to tell a server what permissions I am missing!");
        }
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
        } => {
            ctx.send(|m| {
                m.content(format!(
                    "You are on cooldown try again in {} seconds, moron!",
                    remaining_cooldown.as_secs()
                ))
                .ephemeral(true)
            })
            .await
            .expect("Failed to meme on someone for running a command while on cooldown");
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                event!(Level::WARN, info = "Error while handling error (ironic)", error = ?e);
            }
        }
    }
}

/// Handle events here, should move anything that isn't just println! to a seperate function to avoid the mess that was `on_message` in the python version
#[instrument(skip(ctx, _framework, event, _user_data))]
async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!(
                "{} {}{}",
                "Bot is now online as".color("Purple"),
                data_about_bot.user.name.bright_cyan().bold().underline(),
                "!".color("Purple")
            );

            let activity = Activity::playing(
                "use /help to see all commands. /request_setup to request extra admin features.",
            );
            let status = OnlineStatus::Online;

            // TODO: Store and get from DB so we can change it later and keep it consistent between boots
            ctx.set_presence(Some(activity), status).await;
        }
        poise::Event::Resume { event } => {
            handle_resume(event);
        }
        poise::Event::Message { new_message } => {
            if new_message.is_private() {
                handle_dms(new_message, ctx).await?;
            } else {
                pog_be_gone(new_message, ctx).await?;
            }
        }
        poise::Event::GuildMemberAddition { new_member } => {
            #[cfg(feature = "database")]
            alt_kicker(ctx, new_member).await?;

            #[cfg(feature = "database")]
            bl_warner(ctx, new_member).await?;
        }
        poise::Event::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            handle_msg_delete(guild_id, ctx, channel_id, deleted_message_id).await?;
        }
        poise::Event::MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            if let Some(n) = new.clone() {
                // TODO: put your own guild ID here, this is for tracking message edits
                if !n.is_private() && *event.guild_id.unwrap().as_u64() == 737_168_134_502_350_849 {
                    // I need to learn the overall benifit to using `if let`
                    handle_msg_edit(event.clone(), old_if_available, ctx, new).await?;
                }
            }
        }
        poise::Event::ChannelDelete { channel } => {
            if *channel.guild_id.as_u64() == FBT_GUILD_ID {
                let messages = ctx.cache.channel_messages_field(channel.id.0, |s| {
                    s.filter_map(|m| {
                        if m.channel_id.0 == channel.id.0 {
                            Some(m.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                });

                // let messages = match channel.messages(ctx, |b| b.limit(100)).await {
                //     Ok(vec) => format!("{:?}", vec),
                //     Err(e) => format!("{:?}", e),
                // };

                let shit_list = format!(
                    "Stored channel info:\n\n{:?}\n\nLast 100 messages stored in cache:\n\n{:#?}",
                    channel, messages
                );

                let client = reqwest::Client::new();

                // TODO: I setup a custom paste bin here uhh you can figure out how to reaplce it or just comment out the poise::Event::ChannelDelete event

                let response = client
                    .post("https://paste.buymymojo.net/documents")
                    .body(shit_list)
                    .header("Accept", "application/json")
                    .send()
                    .await?;

                let response_content = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "{'key': 'nope<3'}".to_string());

                let id: PasteResponse = serde_json::from_str(&response_content)?;

                // TODO: channel to alert users that a channel has been deleted

                ChannelId(891_294_507_923_025_951)
                    .send_message(ctx.http.clone(), |f| {
                        f.embed(|e| {
                            e.title("A channel has been deleted".to_string())
                                .field(
                                    "The last cached info from the channel:",
                                    format!("https://paste.buymymojo.net/{}", id.key),
                                    false,
                                )
                                .color(Colour::new(0x00ED_4245))
                        })
                    })
                    .await?;
            }
        }
        poise::Event::CacheReady { guilds } => {
            let args = Args::parse();

            if args.print_guild_cache {
                event!(Level::INFO, "Cache is ready");
            } else {
                event!(Level::INFO, info = "Cache is ready", guilds = ?guilds);
            }
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let console_level = match args.debug {
        DebugLevel::Off => LevelFilter::ERROR,
        DebugLevel::Some => LevelFilter::WARN,
        DebugLevel::Most => LevelFilter::INFO,
        DebugLevel::All => LevelFilter::TRACE,
    };

    let file_level = match args.debug_log {
        LogDebugLevel::Most => LevelFilter::DEBUG,
        LogDebugLevel::All => LevelFilter::TRACE,
    };

    let console_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_ansi(true)
        .with_thread_names(true)
        .with_target(true)
        .with_filter(console_level);
    let file_layer = if args.debug.enabled() {
        match File::create(
            std::path::Path::new(&std::env::current_dir().unwrap()).join(format!(
                "./{}_rusted-fbt.verbose.log",
                chrono::offset::Local::now().timestamp()
            )),
        ) {
            Ok(handle) => {
                let file_log = tracing_subscriber::fmt::layer()
                    .with_line_number(true)
                    .with_ansi(false)
                    .with_thread_names(true)
                    .with_target(true)
                    .with_writer(handle)
                    .with_filter(file_level);
                Some(file_log)
            }
            Err(why) => {
                eprintln!("ERROR!: Unable to create log output file: {why:?}");
                None
            }
        }
    } else {
        None
    };

    let info_file_layer = if args.debug.enabled() {
        match File::create(
            std::path::Path::new(&std::env::current_dir().unwrap()).join(format!(
                "./{}_rusted-fbt.info.log",
                chrono::offset::Local::now().timestamp()
            )),
        ) {
            Ok(handle) => {
                let file_log = tracing_subscriber::fmt::layer()
                    .with_line_number(true)
                    .with_ansi(false)
                    .with_thread_names(true)
                    .with_target(true)
                    .with_writer(handle)
                    .with_filter(LevelFilter::INFO);
                Some(file_log)
            }
            Err(why) => {
                eprintln!("ERROR!: Unable to create log output file: {why:?}");
                None
            }
        }
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(info_file_layer)
        .init();

    // TODO: Like bot admins, put your own IDs here

    let bot_owners: HashSet<UserId> = hashset! {
        UserId::from(212_132_817_017_110_528), // Azuki
        UserId::from(164_694_510_947_794_944), // Cross
        UserId::from(383_507_911_160_233_985), // Mojo
    };

    // * This is where we put the functions that we want in discord
    #[allow(unused_mut)]
    let mut discord_commands = vec![
        about(),
        account_age(),
        ban(),
        botmsg(),
        creation_date(),
        cringe(),
        help(),
        owo(),
        ping(),
        pog(),
        register(),
        shutdown(),
        toxic(),
        waifu(),
        new_ticket(),
        close_ticket(),
        bot_owner_tool_1(),
    ];

    // * Any command that requires the DB goes here
    #[cfg(feature = "database")]
    {
        let mut db_vec = vec![
            add(),
            announcement(),
            authorize(),
            footprint_lookup(),
            excel(),
            feedback(),
            remove_guild(),
            whitelist(),
            request_setup(),
            search(),
            setup(),
            // sqlite_transfer(), // Deprecated
            toggle_kick(),
            update_search_engine(),
            key(),
            invite_info(),
        ];

        discord_commands.append(&mut db_vec);
    }

    // * Any command that are not complete/working here
    #[cfg(feature = "beta")]
    {
        let mut beta_vec = vec![];

        discord_commands.append(&mut beta_vec);
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: discord_commands,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(args.prefix),
                ..Default::default()
            },
            // The global error handler for all error cases that may occur
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, framework, user_data| {
                Box::pin(event_listener(ctx, event, framework, user_data))
            },
            owners: bot_owners,
            // Every command invocation must pass this check to continue execution
            #[cfg(feature = "database")]
            command_check: Some(|ctx| {
                Box::pin(async move {
                    if bot_admin_check(ctx).await.unwrap() {
                        return Ok(true);
                    }

                    let mut con = open_redis_connection().await?;

                    let key_list: HashSet<String> = redis::cmd("SMEMBERS")
                        .arg("user-lists:banned-from-bot")
                        .clone()
                        .query_async(&mut con)
                        .await?;

                    if key_list.contains(&format!("{}", ctx.author().id.as_u64())) {
                        Ok({
                            println!(
                                "{}/{} was blocked from using the bot",
                                ctx.author().id,
                                ctx.author().name
                            );
                            false
                        })
                    } else {
                        Ok(true)
                    }
                })
            }),
            #[cfg(feature = "database")]
            post_command: |_ctx| {
                Box::pin(async move {
                    inc_execution_count().await.expect("");
                })
            },
            ..Default::default()
        })
        .token(args.token)
        .intents(serenity::GatewayIntents::all())
        .setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .client_settings(|f| f.cache_settings(|cs| cs.max_messages(5_000)));

    framework.run_autosharded().await.unwrap();
}
