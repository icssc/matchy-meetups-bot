use crate::discord_helpers::match_members;
use crate::helpers::{format_id, format_pairs, hash_seed};
use crate::helpers::{handle_error, Pairing};
use crate::types::Context;
use anyhow::Result;
use itertools::Itertools;

async fn handle_create_pairing(ctx: Context<'_>, seed_str: String) -> Result<String> {
    let seed = hash_seed(&seed_str);

    let Pairing(pairs, imperfect_matches) = match_members(ctx, seed).await?;
    let pairs_str = format_pairs(&pairs);
    let key = format!(
        "{}_{}",
        seed_str,
        crate::helpers::checksum_matching(seed, &pairs)
    );
    let num_members: usize = pairs.iter().map(|p| p.len()).sum();
    let imperfect_matches_message = if imperfect_matches.is_empty() {
        "All members were matched with new people".to_owned()
    } else {
        format!(
            "The following members could only be matched with people they may have matched with before: {}",
            imperfect_matches.iter().map(format_id).join(", ")
        )
    };
    Ok(format!(
        "{pairs_str}\nTotal paired members: {num_members}\n{imperfect_matches_message}\nTo send this pairing, use this key: `{key}`"
    ))
}

/// Generate a potential pairing of users who have reacted to a message
#[poise::command(
    slash_command,
    track_edits,
    hide_in_help,
    ephemeral,
    required_permissions = "ADMINISTRATOR",
    aliases("pair"),
    on_error = "handle_error"
)]
pub async fn create_pairing(
    ctx: Context<'_>,
    #[description = "A seed to use for the generated pairing (for example, use the current date)."]
    seed: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let resp = handle_create_pairing(ctx, seed)
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("{resp}");
    ctx.say(resp).await?;
    Ok(())
}
