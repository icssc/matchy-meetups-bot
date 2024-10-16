use crate::helpers::log_error;
use crate::pair::pair;
use crate::types::{Context, Error};
use itertools::Itertools;
use poise::futures_util::future::join_all;

/// Generate a potential pairing of users who have reacted to a message
#[poise::command(
    slash_command,
    track_edits,
    hide_in_help,
    ephemeral,
    required_permissions = "ADMINISTRATOR",
    aliases("pair"),
    on_error = "log_error"
)]
pub async fn create_pairing(
    ctx: Context<'_>,
    #[description = "Link to a message with reactions -- a pairing will be made between users who reacted."]
    message_link: String,
) -> Result<(), Error> {
    println!("{message_link}");
    let message_id = message_link
        .split("/")
        .last()
        .unwrap()
        .trim()
        .parse::<u64>();
    let resp = match message_id {
        Ok(id) => {
            let message = ctx.channel_id().message(&ctx, id).await?;

            let reactions =
                join_all(message.reactions.iter().map(|r| {
                    message.reaction_users(&ctx, r.reaction_type.clone(), Some(100), None)
                }))
                .await
                .iter()
                .filter_map(|r| r.as_ref().ok())
                .flatten()
                .map(|u| u.id)
                .collect::<Vec<_>>();
            if reactions.len() <= 1 {
                format!(
                    "Need at least two reactions to create a pairing (message has {} reaction{}).",
                    reactions.len(),
                    if reactions.len() % 2 == 0 { "" } else { "s" }
                )
            } else {
                let pairs = pair(reactions);
                let pairs_str = pairs
                    .iter()
                    .map(|p| p.iter().map(|u| format!("<@{u}>")).join(", "))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{}", pairs_str)
            }
        }
        Err(_) => "Error: unable to parse link.".to_string(),
    };
    println!("{resp}");
    ctx.say(resp).await?;
    Ok(())
}
