use poise::serenity_prelude::{ChannelId, Colour};
use rand::Rng;
use rusted_fbt_lib::{
    types::{Context, Error},
    utils::open_redis_connection,
    vars::{FEEDBACK_CHANNEL_ID, HELP_EXTRA_TEXT, VERSION},
};
#[cfg(feature = "database")]
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::instrument;
use tracing::{event, Level};

#[instrument(skip(ctx))]
#[poise::command(prefix_command, track_edits, slash_command, category = "Info")]
/// Show this help menu
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: HELP_EXTRA_TEXT,
            show_context_menu_commands: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[cfg(feature = "database")]
#[instrument(skip(ctx))]
#[poise::command(slash_command, category = "Info", member_cooldown = 10, ephemeral)]
/// Provide feedback for the bot team to look at!
pub async fn feedback(
    ctx: Context<'_>,
    #[description = "Feedback you want to provide"] feedback: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut con = open_redis_connection().await?;

    redis::cmd("SET")
        .arg(format!(
            "feedback:{}-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ctx.author().id.as_u64(),
            ctx.author().tag()
        ))
        .arg(feedback.clone())
        .clone()
        .query_async(&mut con)
        .await?;

    let colour = &mut rand::thread_rng().gen_range(0..10_000_000);
    ChannelId(FEEDBACK_CHANNEL_ID)
        .send_message(ctx, |f| {
            f.embed(|e| {
                e.title("New feedback!".to_string())
                    .description(feedback)
                    .color(Colour::new(*colour))
                    .author(|a| a.icon_url(ctx.author().face()).name(ctx.author().tag()))
                    .thumbnail("https://media.giphy.com/media/U4sfHXAALLYBQzPcWk/giphy.gif")
            })
        })
        .await?;

    ctx.say("Thank you for the feedback! It has been sent directly to our developers.")
        .await?;

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Info", member_cooldown = 10)]
/// Have some info about the bot
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let guild_count = ctx
        .serenity_context()
        .cache
        .guilds()
        .len()
        .clone()
        .to_string();

    // TODO: change to your own URLs
    let mut fields = vec![
        ("Help page:", "[https://fbtsecurity.fbtheaven.com/](https://fbtsecurity.fbtheaven.com/)", false),
        ("Remove any of your info from the bot:", "[Delete your data](https://fbtsecurity.fbtheaven.com/data-and-privacy-policy#delete-your-data)", false),
        ("Bot version:", VERSION.unwrap_or("unknown"), false),
        ("Server count:", &guild_count, false),
    ];

    // TODO: reduce the ammount of #[cfg(feature = "database")] here!!

    #[cfg(feature = "database")]
    let mut con = open_redis_connection().await?;

    #[cfg(feature = "database")]
    let execution_count: String = redis::cmd("GET")
        .arg("status:commands-executed")
        .clone()
        .query_async(&mut con)
        .await?;

    #[cfg(feature = "database")]
    let mut new_field = vec![(
        "Total commands run since 2.0.18:",
        execution_count.as_str(),
        false,
    )];

    #[cfg(feature = "database")]
    fields.append(&mut new_field);

    // TODO: change to your own URLs
    ctx.send(|f| {
        f.embed(|e| {
            e.title("About")
            .url("https://fbtsecurity.fbtheaven.com/")
            .author(|a| {
                a.name("FBT Staff")
                .url("https://fbtsecurity.fbtheaven.com/")
            })
            .fields(fields)
            .footer(|foot| {
                foot.text("Time mojo spent on V2.0+")
            })
            .image(format!("https://wakatime.com/badge/user/fd57ff6b-f3f1-4957-b9c6-7e09bc3f0559/project/d2f87f17-8c44-4835-b4f6-f0089e52515f.png?rand={}", rand::thread_rng().gen_range(0..1_000_000_000)))
        })
    })
    .await?;

    #[cfg(feature = "database")]
    event!(
        Level::INFO,
        "Total commands run since 2.0.18" = execution_count.parse::<u32>().unwrap() + 1
    );

    Ok(())
}
