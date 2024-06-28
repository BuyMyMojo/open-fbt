use core::time;

use poise::serenity_prelude::{self as serenity, AttachmentType};
use rusted_fbt_lib::enums::WaifuTypes;
use rusted_fbt_lib::types::{Context, Error};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::instrument;
use uwuifier::uwuify_str_sse;

#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Fun", member_cooldown = 15)]
/// This user is cringe
pub async fn cringe(
    ctx: Context<'_>,
    #[description = "Optionally call this user cringe"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let camera_message = ctx.say("<a:camera:870459823907553352>").await?;

    sleep(time::Duration::from_secs(1)).await;

    camera_message
        .edit(ctx, |b| {
            b.content("<a:camera_with_flash:870458599325986898>")
        })
        .await?;

    sleep(time::Duration::from_secs(1)).await;

    camera_message
        .edit(ctx, |b| b.content("<a:camera:870459823907553352>"))
        .await?;

    match user {
        None => {
            ctx.say("Yep, that's going in my cringe compilation")
                .await?;
        }
        Some(user) => {
            ctx.send(|m| {
                m.content(format!(
                    "Yep <@{}>, that's going in my cringe compilation",
                    user.id
                ))
            })
            .await?;
        }
    }

    Ok(())
}

/// OwOifys your message
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Fun")]
pub async fn owo(ctx: Context<'_>, #[description = "Message"] msg: String) -> Result<(), Error> {
    ctx.say(uwuify_str_sse(msg.as_str())).await?;

    Ok(())
}

/// Replies with pog pog pog and pog frog!
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Fun", member_cooldown = 10)]
pub async fn pog(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|f| {
        f.content("Pog pog pog!")
            .ephemeral(false)
            .attachment(AttachmentType::Bytes {
                data: std::borrow::Cow::Borrowed(include_bytes!("../../assets/pog-frog.gif")),
                filename: String::from("pog-frog.gif"),
            })
    })
    .await?;

    Ok(())
}

/// Sends a random waifu (SFW)
#[instrument(skip(ctx))]
#[poise::command(slash_command, category = "Fun", member_cooldown = 5)]
pub async fn waifu(
    ctx: Context<'_>,
    #[description = "What waifu do you want?"] waifu_type: Option<WaifuTypes>,
) -> Result<(), Error> {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Waifu {
        url: String,
    }

    let choice: String = match waifu_type {
        None => "waifu".to_string(),
        Some(WaifuTypes::Neko) => "neko".to_string(),
        Some(WaifuTypes::Megumin) => "megumin".to_string(),
        Some(WaifuTypes::Bully) => "bully".to_string(),
        Some(WaifuTypes::Cuddle) => "cuddle".to_string(),
        Some(WaifuTypes::Cry) => "cry".to_string(),
        Some(WaifuTypes::Kiss) => "kiss".to_string(),
        Some(WaifuTypes::Lick) => "lick".to_string(),
        Some(WaifuTypes::Pat) => "pat".to_string(),
        Some(WaifuTypes::Smug) => "smug".to_string(),
        Some(WaifuTypes::Bonk) => "bonk".to_string(),
        Some(WaifuTypes::Blush) => "blush".to_string(),
        Some(WaifuTypes::Smile) => "smile".to_string(),
        Some(WaifuTypes::Wave) => "wave".to_string(),
        Some(WaifuTypes::Highfive) => "highfive".to_string(),
        Some(WaifuTypes::Handhold) => "handhold".to_string(),
        Some(WaifuTypes::Nom) => "nom".to_string(),
        Some(WaifuTypes::Bite) => "bite".to_string(),
        Some(WaifuTypes::Glomp) => "glomp".to_string(),
        Some(WaifuTypes::Slap) => "slap".to_string(),
        Some(WaifuTypes::Kill) => "kill".to_string(),
        Some(WaifuTypes::Happy) => "happy".to_string(),
        Some(WaifuTypes::Wink) => "wink".to_string(),
        Some(WaifuTypes::Poke) => "poke".to_string(),
        Some(WaifuTypes::Dance) => "dance".to_string(),
        Some(WaifuTypes::Cringe) => "cringe".to_string(),
    };

    let response = reqwest::get(format!("https://api.waifu.pics/sfw/{choice}")).await?;
    let waifu: Waifu = response.json().await?;

    // let waifu: Waifu = serde_json::from_str(json_content.as_str())?;

    ctx.send(|b| b.embed(|e| e.title("Your waifu:").image(waifu.url)))
        .await?;

    Ok(())
}

/// Replies with pong!
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Fun", member_cooldown = 15)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong! 🏓").await?;

    Ok(())
}

/// That's toxic
#[instrument(skip(ctx))]
#[poise::command(slash_command, category = "Fun", member_cooldown = 15)]
pub async fn toxic(
    ctx: Context<'_>,
    #[description = "Optionally call this user cringe"] user: Option<serenity::User>,
) -> Result<(), Error> {
    const GIF_NAME: &str = "toxic.gif";
    let toxic_gif = include_bytes!("../../assets/toxic.gif");

    ctx.defer().await?;

    match user {
        None => {
            ctx.send(|m| {
                m.content("That's toxic!")
                    .attachment(AttachmentType::Bytes {
                        data: std::borrow::Cow::Borrowed(toxic_gif), // include_bytes! directly embeds the gif file into the executable at compile time.
                        filename: GIF_NAME.to_string(),
                    })
            })
            .await?;
        }
        Some(user) => {
            ctx.send(|m| {
                m.content(format!("<@{}>, That's toxic!", user.id.as_u64()))
                    .attachment(AttachmentType::Bytes {
                        data: std::borrow::Cow::Borrowed(toxic_gif), // include_bytes! directly embeds the gif file into the executable at compile time.
                        filename: GIF_NAME.to_string(),
                    })
            })
            .await?;
        }
    }

    Ok(())
}
