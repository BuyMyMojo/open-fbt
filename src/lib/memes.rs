use crate::vars::FBT_GUILD_ID;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{self as serenity};
use regex::Regex;
use strip_markdown::strip_markdown;

// Some times maybe good sometimes maybe shit
pub static POG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"[pP𝓹⍴𝖕𝔭የ𝕡ק🅟🅿ⓟρᑭ𝙥քp̷þp͎₱ᵽ℘ｱ𝐩𝒑𝓅p̞̈͑̚͞℘p͓̽ք𝓹ᕶp̶p̳p̅][oO0øØ𝓸᥆𝖔𝔬ዐ𝕠๏🅞🅾ⓞσOｏÒօoo̷ðo͎の𝗼ᴏᵒ🇴‌𝙤ѻⲟᓍӨo͓̽o͟o̲o̅o̳o̶🄾o̯̱̊͊͢Ꭷσℴ𝒐𝐨][gG9𝓰𝖌𝔤𝕘🅖🅶ⓖɠGｇ𝑔ցg̷gg͎g̲g͟ǥ₲ɢg͓̽Gɠ𝓰𝙜🇬‌Ꮆᵍɢ𝗴𝐠𝒈𝑔ᧁg𝚐₲Ꮆ𝑔ĝ̽̓̀͑𝘨ງ🄶𝔤ģ]\b",
    ).unwrap()
});

/// This will read a message and check to see if the message contains the word `pog`
///
/// # Panics
///
/// Panics if regex fails to compile, this should be unreachable unless I acidentally change something before compile time.
///
/// # Errors
///
/// This function will return an error if .
pub async fn pog_be_gone(
    new_message: &serenity::Message,
    ctx: &serenity::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !new_message.author.bot && !new_message.content.is_empty() {
        match new_message.guild(ctx) {
            None => {} // Probably a DM, do nothing
            Some(guild) => {
                if guild.id.as_u64() == &FBT_GUILD_ID {
                    let lowercase_message = new_message.content.to_lowercase();
                    let cleaned_message = strip_markdown(&lowercase_message);

                    let words: Vec<&str> = cleaned_message.split(' ').collect();
                    let mut hits: Vec<&str> = Vec::new();

                    for word in words {
                        let _ = POG_RE.find(word).map_or((), |pog| {
                            hits.push(pog.as_str());
                        });
                    }

                    if !hits.is_empty() {
                        // there is at least 1 pog found
                        if hits.capacity().gt(&10) {
                            new_message
                                .reply(
                                    ctx,
                                    format!(
                                        "Jesus dude, why did you pog {} times?! stop it!",
                                        hits.len()
                                    ),
                                )
                                .await?;
                        } else {
                            new_message.reply_mention(ctx, "please refer to the rules and use the term 'poi' instead of 'pog'!").await?;
                        }
                    }
                }
            }
        }
    };
    Ok(())
}

#[cfg(test)]
mod meme_tests {
    use super::*;

    #[test]
    fn test_regex() {
        let pog_test = "pog";

        assert!(POG_RE.is_match(pog_test));
    }
}
